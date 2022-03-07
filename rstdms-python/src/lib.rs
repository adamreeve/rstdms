use chrono::{Datelike, Timelike};
use std::fs::File;
use std::io::{Read, Seek};
use std::sync::Arc;
use std::{error, fmt};

use arrow2::array::{Array, PrimitiveArray};
use arrow2::datatypes::Field;
use arrow2::ffi::{export_array_to_c, export_field_to_c, Ffi_ArrowArray, Ffi_ArrowSchema};
use arrow2::types::NativeType as ArrowNativeType;
use pyo3::exceptions::{PyIOError, PyNotImplementedError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDateTime, PyDict};
use rstdms::timestamp::Timestamp;
use rstdms::{Channel, NativeType, TdmsFile, TdmsReadError, TdmsValue};

#[pyclass(name = "InternalTdmsFile")]
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

    fn channel_data(
        &self,
        group_name: &str,
        channel_name: &str,
        schema_ptr_in: usize,
        array_ptr_in: usize,
    ) -> PyResult<()> {
        let schema_ptr = schema_ptr_in as *mut Ffi_ArrowSchema;
        let array_ptr = array_ptr_in as *mut Ffi_ArrowArray;
        match self.inner.group(group_name) {
            Some(group) => match group.channel(channel_name) {
                Some(channel) => match channel.data_type() {
                    rstdms::TdsType::Void => Err(PyValueError::new_err("channel has no data type")),
                    rstdms::TdsType::I8 => {
                        read_channel_data::<i8, _>(&channel, schema_ptr, array_ptr)
                    }
                    rstdms::TdsType::I16 => {
                        read_channel_data::<i16, _>(&channel, schema_ptr, array_ptr)
                    }
                    rstdms::TdsType::I32 => {
                        read_channel_data::<i32, _>(&channel, schema_ptr, array_ptr)
                    }
                    rstdms::TdsType::I64 => {
                        read_channel_data::<i64, _>(&channel, schema_ptr, array_ptr)
                    }
                    rstdms::TdsType::U8 => {
                        read_channel_data::<u8, _>(&channel, schema_ptr, array_ptr)
                    }
                    rstdms::TdsType::U16 => {
                        read_channel_data::<u16, _>(&channel, schema_ptr, array_ptr)
                    }
                    rstdms::TdsType::U32 => {
                        read_channel_data::<u32, _>(&channel, schema_ptr, array_ptr)
                    }
                    rstdms::TdsType::U64 => {
                        read_channel_data::<u64, _>(&channel, schema_ptr, array_ptr)
                    }
                    rstdms::TdsType::SingleFloat => {
                        read_channel_data::<f32, _>(&channel, schema_ptr, array_ptr)
                    }
                    rstdms::TdsType::DoubleFloat => {
                        read_channel_data::<f64, _>(&channel, schema_ptr, array_ptr)
                    }
                    rstdms::TdsType::ExtendedFloat => Err(PyNotImplementedError::new_err(
                        "Reading ExtendedFloat data is not implemented",
                    )),
                    rstdms::TdsType::SingleFloatWithUnit => {
                        read_channel_data::<f32, _>(&channel, schema_ptr, array_ptr)
                    }
                    rstdms::TdsType::DoubleFloatWithUnit => {
                        read_channel_data::<f64, _>(&channel, schema_ptr, array_ptr)
                    }
                    rstdms::TdsType::ExtendedFloatWithUnit => Err(PyNotImplementedError::new_err(
                        "Reading ExtendedFloat data is not implemented",
                    )),
                    rstdms::TdsType::String => Err(PyNotImplementedError::new_err(
                        "Reading String data is not implemented",
                    )),
                    rstdms::TdsType::Boolean => Err(PyNotImplementedError::new_err(
                        "Reading Boolean data is not implemented",
                    )),
                    rstdms::TdsType::TimeStamp => Err(PyNotImplementedError::new_err(
                        "Reading TimeStamp data is not implemented",
                    )),
                    rstdms::TdsType::FixedPoint => Err(PyNotImplementedError::new_err(
                        "Reading FixedPoint data is not implemented",
                    )),
                    rstdms::TdsType::ComplexSingleFloat => Err(PyNotImplementedError::new_err(
                        "Reading ComplexSingleFloat data is not implemented",
                    )),
                    rstdms::TdsType::ComplexDoubleFloat => Err(PyNotImplementedError::new_err(
                        "Reading ComplexDoubleFloat data is not implemented",
                    )),
                    rstdms::TdsType::DaqmxRawData => Err(PyNotImplementedError::new_err(
                        "Reading DaqmxRawData is not implemented",
                    )),
                },
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

fn read_channel_data<T, TFile: Read + Seek>(
    channel: &Channel<TFile>,
    schema_ptr: *mut Ffi_ArrowSchema,
    array_ptr: *mut Ffi_ArrowArray,
) -> PyResult<()>
where
    T: NativeType + ArrowNativeType,
    TFile: Read + Seek,
{
    let len = channel.len();
    let mut data: Vec<T> = vec![Default::default(); len as usize];
    channel
        .read_all_data(&mut data)
        .map_err(PyTdmsError::from)
        .map_err(PyErr::from)?;
    let array: Arc<dyn Array> = Arc::new(PrimitiveArray::from_vec(data));
    let field = Field::new("data", array.data_type().clone(), false);
    unsafe {
        export_field_to_c(&field, schema_ptr);
        export_array_to_c(array, array_ptr);
    }
    Ok(())
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
        match err {
            PyTdmsError::TdmsReadError(TdmsReadError::IoError(_)) => PyIOError::new_err(err.to_string()),
            PyTdmsError::TdmsReadError(_) => PyValueError::new_err(err.to_string()),
        }
    }
}
