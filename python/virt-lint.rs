/* SPDX-License-Identifier: LGPL-3.0-or-later */

use ::virt_lint::*;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::sync::{Arc, Mutex};
use virt::connect::Connect;
use virt::sys;

struct PyVirtLintError(errors::VirtLintError);

impl From<PyVirtLintError> for PyErr {
    fn from(error: PyVirtLintError) -> Self {
        PyValueError::new_err(error.0.to_string())
    }
}

impl From<errors::VirtLintError> for PyVirtLintError {
    fn from(other: errors::VirtLintError) -> Self {
        Self(other)
    }
}

#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PyVirtLintWarning {
    tags: Vec<String>,
    domain: WarningDomain,
    level: WarningLevel,
    msg: String,
}

#[pymethods]
impl PyVirtLintWarning {
    fn __repr__(slf: Bound<'_, Self>) -> PyResult<String> {
        let class_name = slf.get_type().name()?;
        Ok(format!("{}({:?})", class_name, slf.borrow()))
    }
}

impl From<&VirtLintWarning> for PyVirtLintWarning {
    fn from(other: &VirtLintWarning) -> Self {
        let (tags, domain, level, msg) = other.get();
        Self {
            tags: tags.to_vec(),
            domain: *domain,
            level: *level,
            msg: msg.to_string(),
        }
    }
}

#[pyclass(name = "VirtLint")]
pub struct PyVirtLint {
    vl: Arc<Mutex<VirtLint>>,
}

#[pymethods]
impl PyVirtLint {
    #[new]
    #[pyo3(signature = (conn=None))]
    fn new(conn: Option<PyObject>) -> Self {
        let rust_conn = match conn {
            None => None,
            Some(x) => {
                let c: usize = Python::with_gil(|py| {
                    let locals = [("conn", x)].into_py_dict_bound(py);
                    let code = "conn.c_pointer()";
                    py.eval_bound(code, None, Some(&locals))?.extract()
                })
                .expect("ble");

                Some(unsafe { Connect::from_ptr(c as sys::virConnectPtr) })
            }
        };

        let vl = Arc::new(Mutex::new(VirtLint::new(rust_conn.as_ref())));
        Self { vl }
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let class_name = slf.get_type().name()?;
        Ok(format!("{}({:?})", class_name, slf.borrow().vl))
    }

    #[pyo3(signature = (capsxml = None))]
    fn capabilities_set(&mut self, capsxml: Option<String>) -> PyResult<()> {
        self.vl.lock().unwrap().capabilities_set(capsxml)?;
        Ok(())
    }

    fn domain_capabilities_clear(&mut self) {
        self.vl.lock().unwrap().domain_capabilities_clear();
    }

    fn domain_capabilities_add(&mut self, domcapsxml: String) -> PyResult<()> {
        self.vl
            .lock()
            .unwrap()
            .domain_capabilities_add(domcapsxml)?;

        Ok(())
    }

    fn validate(
        &mut self,
        domxml: &str,
        validator_tags: Vec<String>,
        error_on_no_connect: bool,
    ) -> PyResult<()> {
        self.vl
            .lock()
            .unwrap()
            .validate(domxml, &validator_tags, error_on_no_connect)?;

        Ok(())
    }

    fn warnings(&self) -> Vec<PyVirtLintWarning> {
        self.vl
            .lock()
            .unwrap()
            .warnings()
            .iter()
            .map(PyVirtLintWarning::from)
            .collect()
    }

    #[staticmethod]
    fn list_validator_tags() -> PyResult<Vec<String>> {
        Ok(VirtLint::list_validator_tags()?)
    }
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn virt_lint(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<PyVirtLint>()?;
    Ok(())
}
