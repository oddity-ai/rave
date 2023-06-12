pub trait Device {
    type Container<T>;
}

pub struct Local;

impl Device for Local {
    type Container<T> = T;
}

pub struct Cuda;

impl Device for Cuda {
    type Container<T> = *const std::ffi::c_void;
}
