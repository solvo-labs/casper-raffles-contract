use casper_types::ApiError;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum Error {
    FatalError = 0,
    AdminError = 1,
    TimeError = 2,
    WinnerError = 3,
    NotApproved = 4,
    CancelError = 5,
    DrawExists = 6,
    WinnerAlreadyExist = 7,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}
