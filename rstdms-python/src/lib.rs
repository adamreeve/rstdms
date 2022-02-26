use chrono::{Datelike, Timelike};
use std::fs::File;
use std::{error, fmt};

use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDateTime, PyDict};
use rstdms::timestamp::Timestamp;
use rstdms::{TdmsFile, TdmsReadError, TdmsValue};

#[pyclass(name = "TdmsFile")]
struct PyTdmsFile {
    inner: TdmsFile<File>,
}

#[pyclass]
struct TdmsTimestamp {
    #[pyo3(get)]
    second_fractions: u64,
    #[pyo3(get)]
    seconds: i64,
}

#[pymethods]
impl PyTdmsFile {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let file = File::open(path)?;
        let tdms_file = TdmsFile::new(file).map_err(PyTdmsError::from)?;
        Ok(PyTdmsFile { inner: tdms_file })
    }

    fn groups(&self) -> Vec<String> {
        self.inner.groups().map(|g| g.name().to_owned()).collect()
    }

    fn group_channels(&self, group_name: &str) -> PyResult<Vec<String>> {
        match self.inner.group(group_name) {
            Some(group) => Ok(group.channels().map(|c| c.name().to_owned()).collect()),
            None => Err(PyValueError::new_err(format!(
                "Invalid group name '{}'",
                group_name
            ))),
        }
    }

    fn properties(&self) -> PyResult<Py<PyAny>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let dict = PyDict::new(py);
        for property in self.inner.properties() {
            dict.set_item(&property.name, to_py_object(&py, &property.value))?;
        }
        Ok(dict.to_object(py))
    }

    fn group_properties(&self, group_name: &str) -> PyResult<Py<PyAny>> {
        match self.inner.group(group_name) {
            Some(group) => {
                let gil = Python::acquire_gil();
                let py = gil.python();
                let dict = PyDict::new(py);
                for property in group.properties() {
                    dict.set_item(&property.name, to_py_object(&py, &property.value))?;
                }
                Ok(dict.to_object(py))
            }
            None => Err(PyValueError::new_err(format!(
                "Invalid group name '{}'",
                group_name
            ))),
        }
    }

    fn channel_properties(&self, group_name: &str, channel_name: &str) -> PyResult<Py<PyAny>> {
        match self.inner.group(group_name) {
            Some(group) => match group.channel(channel_name) {
                Some(channel) => {
                    let gil = Python::acquire_gil();
                    let py = gil.python();
                    let dict = PyDict::new(py);
                    for property in channel.properties() {
                        dict.set_item(&property.name, to_py_object(&py, &property.value))?;
                    }
                    Ok(dict.to_object(py))
                }
                None => Err(PyValueError::new_err(format!(
                    "Invalid channel name '{}'",
                    channel_name
                ))),
            },
            None => Err(PyValueError::new_err(format!(
                "Invalid group name '{}'",
                group_name
            ))),
        }
    }
}

impl TdmsTimestamp {
    pub fn from_timestamp(timestamp: &Timestamp) -> TdmsTimestamp {
        TdmsTimestamp {
            seconds: timestamp.seconds,
            second_fractions: timestamp.second_fractions,
        }
    }
}

#[pymethods]
impl TdmsTimestamp {
    fn to_datetime(&self) -> PyResult<Py<PyAny>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let timestamp = Timestamp::new(self.seconds, self.second_fractions);
        match timestamp.to_datetime() {
            Some(datetime) => {
                let month: u8 = datetime.month().try_into()?;
                let day: u8 = datetime.day().try_into()?;
                let hour: u8 = datetime.hour().try_into()?;
                let minute: u8 = datetime.minute().try_into()?;
                let second: u8 = datetime.second().try_into()?;
                PyDateTime::new(
                    py,
                    datetime.year(),
                    month,
                    day,
                    hour,
                    minute,
                    second,
                    datetime.nanosecond() / 1000u32,
                    None,
                )
                .map(|dt| dt.into_py(py))
            }
            None => Err(PyValueError::new_err("Invalid timestamp")),
        }
    }
}

fn to_py_object(py: &Python, value: &TdmsValue) -> Py<PyAny> {
    match value {
        TdmsValue::Int8(value) => value.into_py(*py),
        TdmsValue::Int16(value) => value.into_py(*py),
        TdmsValue::Int32(value) => value.into_py(*py),
        TdmsValue::Int64(value) => value.into_py(*py),
        TdmsValue::Uint8(value) => value.into_py(*py),
        TdmsValue::Uint16(value) => value.into_py(*py),
        TdmsValue::Uint32(value) => value.into_py(*py),
        TdmsValue::Uint64(value) => value.into_py(*py),
        TdmsValue::Float32(value) => value.into_py(*py),
        TdmsValue::Float64(value) => value.into_py(*py),
        TdmsValue::String(value) => value.into_py(*py),
        TdmsValue::Timestamp(value) => TdmsTimestamp::from_timestamp(value).into_py(*py),
    }
}

/// Reads TDMS file data
#[pymodule]
fn rstdms(_py: Python, m: &PyModule) -> PyResult<()> {
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
