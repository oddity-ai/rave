pub trait Device {}

pub struct Local;

impl Device for Local {}

pub struct Cuda;

impl Device for Cuda {}
