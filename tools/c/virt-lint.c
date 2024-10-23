/* SPDX-License-Identifier: LGPL-3.0-or-later */

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <getopt.h>
#include <string.h>
#include <stdbool.h>
#include <errno.h>
#include <libgen.h>

#include <libvirt/libvirt.h>
#include <libvirt/virterror.h>
#include <virt_lint.h>

#define NULLSTR(x) (x ?: "<null>")

#define STR(x) #x

#define ARRAY_CARDINALITY(x) (sizeof(x)/sizeof(*x))

#define ERROR(...) \
    do { \
        fprintf(stderr, "ERROR %s:%d : ", __FUNCTION__, __LINE__); \
        fprintf(stderr, __VA_ARGS__); \
        fprintf(stderr, "\n"); \
    } while (0)

#define ERROR_OOM() \
    do { \
        ERROR("Out of memory"); \
        abort(); \
    } while (0)

#define ENUM_2_STR(EnumTyp, ...) \
    static const char * EnumTyp ## ToStr(unsigned int x) { \
        const char *vals[] = { __VA_ARGS__ }; \
        unsigned int max = ARRAY_CARDINALITY(vals); \
        return x < max ? vals[x] : NULL; \
    }

ENUM_2_STR(WarningDomain,
           STR(Domain),
           STR(Node),
);

ENUM_2_STR(WarningLevel,
           STR(Error),
           STR(Warning),
           STR(Notice),
);

static void
append_string(char ***array,
              size_t *size,
              const char *str)
{
    if (!(*array = realloc(*array, (*size + 1) * sizeof(str)))) {
        ERROR_OOM();
    }

    if (!((*array)[*size] = strdup(str))) {
        ERROR_OOM();
    }
    (*size)++;
}

static void
parse_list(char ***array,
           size_t *size,
           const char *str)
{
    char *copy = strdup(str);
    char *saveptr = NULL;
    char *tokstr = NULL;
    const char *delim = ",";
    const char *ble;

    if (!copy) {
        ERROR_OOM();
    }

    tokstr = copy;

    while (1) {
        char *token = strtok_r(tokstr, delim, &saveptr);

        if (token == NULL)
            break;

        append_string(array, size, token);
        tokstr = NULL;
    }

    free(copy);
}

static ssize_t
read_contents(FILE *fp,
              const char *path,
              char **content)
{
    size_t content_size = 0;

    *content = NULL;

    while (!feof(fp)) {
        char buf[256] = { 0 };
        size_t nread = 0;

        nread = fread(buf, sizeof(*buf), ARRAY_CARDINALITY(buf),  fp);
        if (ferror(fp)) {
            ERROR("Unable to read from file '%s'", path);
            return -1;
        }

        if (nread == 0) {
            break;
        }

        *content = realloc(*content, content_size + nread + 1);
        if (!*content) {
            ERROR_OOM();
        }

        memcpy(*content + content_size, buf, nread);
        content_size += nread;
        (*content)[content_size] = '\0';
    }

    return content_size;
}

static ssize_t
read_file(const char *path,
          char **content)
{
    FILE *fp = fopen(path, "r");
    ssize_t ret = -1;

    if (!fp) {
        ERROR("Unable to open file '%s': %s", path, strerror(errno));
        return -1;
    }

    ret = read_contents(fp, path, content);
    fclose(fp);
    return ret;
}

static void
list_validator_tags(void)
{
    struct VirtLintError *vlErr = NULL;
    char **tags = NULL;
    ssize_t ntags = 0;
    size_t i = 0;

    if ((ntags = virt_lint_list_tags(&tags, &vlErr)) < 0) {
        char *msg = virt_lint_error_get_message(vlErr);

        ERROR("Unable to list tags: %s", msg);
        virt_lint_string_free(msg);
        virt_lint_error_free(&vlErr);
        return;
    }

    for (i = 0; i < ntags; i++) {
        printf("%s\n", tags[i]);
        virt_lint_string_free(tags[i]);
    }

    virt_lint_string_free((char*)tags);
}

static void
print_help(const char *progname)
{
    char *progname_dup = strdup(progname);
    char *base;

    if (!progname_dup) {
        ERROR_OOM();
    }

    base = basename(progname_dup);

    printf("Virtualization linting library\n"
           "\n"
           "Usage: %s [OPTIONS]\n"
           "\n"
           "Options:\n"
           "  -c, --connect <URI>            connection uri\n"
           "  -p, --path <FILE>              The path to the domain XML, otherwise read the XML from stdin\n"
           "  -d, --debug                    Turn debugging information on\n"
           "  -v, --validators <VALIDATORS>  Comma separated list of validator tags, empty means all\n"
           "  -l, --list-validator-tags      List known validator tags\n"
           "  -h, --help                     Print help\n"
           "  -V, --version                  Print version\n",
           base);

    free(progname_dup);
}

static void
print_version(void)
{
    unsigned long version = virt_lint_version();

    printf("virt-lint: %lu.%lu.%lu\n", version / 1000000, version / 1000, version % 1000);
}

static void
clippy(const char *progname)
{
    if (!strstr(progname, "clippy"))
        return;

    printf("/‾‾\\\n|  |\n@  @\n|| |/\n|| ||\n|\\_/|\n\\___/\n  /\\\n"
           "/‾  ‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾\\\n"
           "| It looks like you're linting some libvirt  |\n"
           "| XMLs. Would you like some help with that?  |\n"
           "\\____________________________________________/\n");
}

typedef struct {
    const char *uri;
    const char *path;
    int debug;
    char **tags;
    size_t ntags;
} Cli;

static int
parse_args(Cli *cli,
           int argc,
           char *argv[])
{
    int arg;
    struct option opt[] = {
        { "connect", required_argument, NULL, 'c' },
        { "path", required_argument, NULL, 'p' },
        { "debug", no_argument, NULL, 'd' },
        { "validators", required_argument, NULL, 'v' },
        { "list-validator-tags", no_argument, NULL, 'l' },
        { "help", no_argument, NULL, 'h' },
        { "version", no_argument, NULL, 'V' },
        { NULL, 0, NULL, 0 },
    };

    memset(cli, 0, sizeof(*cli));

    clippy(argv[0]);

    while ((arg = getopt_long(argc, argv, "+:c:p:dv:lhV", opt, NULL)) != -1) {
        switch (arg) {
        case 'c':
            cli->uri = optarg;
            break;
        case 'p':
            cli->path = optarg;
            break;
        case 'd':
            cli->debug = true;
            break;
        case 'v':
            parse_list(&cli->tags, &cli->ntags, optarg);
            break;
        case 'l':
            list_validator_tags();
            exit(EXIT_SUCCESS);
        case 'h':
            print_help(argv[0]);
            exit(EXIT_SUCCESS);
        case 'V':
            print_version();
            exit(EXIT_SUCCESS);
        }
    }

    return 0;
}

static void
free_args(Cli *cli)
{
    size_t i;

    for (i = 0; i < cli->ntags; i++) {
        free(cli->tags[i]);
    }

    free(cli->tags);
}

static int
virt_lint_worker(virConnectPtr conn,
                 const char *xml,
                 const char **tags,
                 size_t ntags)
{
    struct VirtLint *vl = NULL;
    struct VirtLintError *vlErr = NULL;
    struct CVirtLintWarning *warnings = NULL;
    ssize_t nwarnings = 0;
    size_t i;
    int ret = -1;

    if (!(vl = virt_lint_new(conn))) {
        ERROR_OOM();
        goto cleanup;
    }

    if (virt_lint_validate(vl, xml, tags, ntags, false, &vlErr) < 0) {
        char *msg = virt_lint_error_get_message(vlErr);

        ERROR("Validation failed: %s", msg);
        virt_lint_string_free(msg);
        goto cleanup;
    }

    if ((nwarnings = virt_lint_get_warnings(vl, &warnings, &vlErr)) < 0) {
        char *msg = virt_lint_error_get_message(vlErr);

        ERROR("Unable to get warnings: %s", msg);
        virt_lint_string_free(msg);
        goto cleanup;
    }

    for (i = 0; i < nwarnings; i++) {
        struct CVirtLintWarning *w = &warnings[i];
        size_t i;

        printf("Warning: tags=[");
        for (i = 0; i < w->ntags; i++) {
            if (i > 0) {
                printf(", ");
            }
            printf("\"%s\"", w->tags[i]);
        }
        printf("]\t");

        printf("domain=%s\tlevel=%s\tmsg=%s\n",
               NULLSTR(WarningDomainToStr(w->domain)),
               NULLSTR(WarningLevelToStr(w->level)),
               NULLSTR(w->msg));
    }

    ret = 0;
 cleanup:
    virt_lint_warnings_free(&warnings, &nwarnings);
    virt_lint_error_free(&vlErr);
    virt_lint_free(vl);
    return ret;
}

int
main(int argc,
     char *argv[])
{
    Cli cli;
    virConnectPtr conn = NULL;
    char *domxml = NULL;
    int ret = EXIT_FAILURE;

    if (parse_args(&cli, argc, argv) < 0) {
        goto cleanup;
    }

    if (cli.path) {
        if (read_file(cli.path, &domxml) < 0) {
            goto cleanup;
        }
    } else {
        if (read_contents(stdin, "stdin", &domxml) < 0) {
            goto cleanup;
        }
    }

    if (!(conn = virConnectOpen(cli.uri))) {
        fprintf(stderr, "Unable to connect.\n");
        goto cleanup;
    }

    if (virt_lint_worker(conn,
                         domxml,
                         (const char **) cli.tags,
                         cli.ntags) < 0) {
        goto cleanup;
    }

    ret = EXIT_SUCCESS;
 cleanup:
    free(domxml);
    if (conn)
        virConnectClose(conn);
    free_args(&cli);

    return ret;
}
