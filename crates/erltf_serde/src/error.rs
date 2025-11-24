// Copyright (C) 2025-2026 Michael S. Klishin and Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::{de, ser};
use std::fmt::Display;
use std::result::Result as StdResult;
use thiserror::Error;

pub type Result<T> = StdResult<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("erltf error: {0}")]
    Erltf(#[from] erltf::errors::Error),

    #[error("{0}")]
    Message(String),

    #[error("unsupported type: {0}")]
    UnsupportedType(String),

    #[error("expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("invalid value: {0}")]
    InvalidValue(String),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
