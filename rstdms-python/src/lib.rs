extern crate rstdms;

use std::{error, fmt};
use std::fs::File;

use rstdms::{TdmsFile, TdmsReadError};
use pyo3::prelude::*;
use pyo3::exceptions::PyException;

#[pyclass(name="TdmsFile")]
struct PyTdmsFile {
    _inner: TdmsFile<File>
}

#[pymethods]
impl PyTdmsFile {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let file = File::open(path)?;
        let tdms_file = TdmsFile::new(file).map_err(PyTdmsError::from)?;
        Ok(PyTdmsFile { _inner: tdms_file })
    }
}

/// Reads TDMS file data
#[pymodule]
fn rstdms_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTdmsFile>()?;
    Ok(())
}

#[derive(Debug)]
enum PyTdmsError {
    TdmsReadError(TdmsReadError),
}

impl fmt::Display for PyTdmsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PyTdmsError::TdmsReadError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for PyTdmsError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            PyTdmsError::TdmsReadError(ref e) => Some(e),
        }
    }
}

impl From<TdmsReadError> for PyTdmsError {
    fn from(err: TdmsReadError) -> PyTdmsError {
        PyTdmsError::TdmsReadError(err)
    }
}

impl From<PyTdmsError> for PyErr {
    fn from(err: PyTdmsError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}
