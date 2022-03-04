import pyarrow as pa
from pyarrow.cffi import ffi

from .rstdms import InternalTdmsFile


class TdmsFile:
    def __init__(self, path):
        self._file = InternalTdmsFile(path)
        self._groups = set(self._file.groups())

    def __getitem__(self, group_name):
        if group_name in self._groups:
            return TdmsGroup(self._file, group_name)
        raise KeyError(f"No group named '{group_name}' found")


class TdmsGroup:
    def __init__(self, file, group_name):
        self._file = file
        self._group_name = group_name
        self._channels = set(self._file.group_channels(self._group_name))

    def __getitem__(self, channel_name):
        if channel_name in self._channels:
            return TdmsChannel(self._file, self._group_name, channel_name)
        raise KeyError(
                f"No channel named '{channel_name}' found in group "
                f"'{self._group_name}'")


class TdmsChannel:
    def __init__(self, file, group_name, channel_name):
        self._file = file
        self._group_name = group_name
        self._channel_name = channel_name

    def read_all_data(self):
        c_schema = ffi.new("struct ArrowSchema*")
        ptr_schema = int(ffi.cast("uintptr_t", c_schema))
        c_array = ffi.new("struct ArrowArray*")
        ptr_array = int(ffi.cast("uintptr_t", c_array))
        self._file.channel_data(
                self._group_name, self._channel_name, ptr_schema, ptr_array)
        return pa.Array._import_from_c(ptr_array, ptr_schema)
