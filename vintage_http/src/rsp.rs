use serde::Serialize;

pub(crate) const RSP_CODE_SUCCESS: i32 = 0;
pub(crate) const RSP_CODE_FAILED: i32 = -1;

pub(crate) const RSP_MSG_SUCCESS: &str = "success";
pub(crate) const RSP_MSG_FAILED: &str = "failed";

#[derive(Serialize)]
pub(crate) struct Rsp<'a, TData>
where
    TData: Serialize,
{
    pub code: i32,
    pub msg: &'a str,
    pub data: Option<TData>,
}

impl<'a, TData> Rsp<'a, TData>
where
    TData: Serialize,
{
    pub fn success(data: TData) -> Self {
        Self {
            code: RSP_CODE_SUCCESS,
            msg: RSP_MSG_SUCCESS,
            data: Some(data),
        }
    }

    pub fn failed() -> Self {
        Self {
            code: RSP_CODE_FAILED,
            msg: RSP_MSG_FAILED,
            data: None,
        }
    }
}
