pub struct Envelope<T> {
    pub to: Option<u32>,
    pub from: Option<u32>,
    pub payload: T,
}

impl<T> Envelope<T> {
    pub fn broadcast(payload: impl Into<T>) -> Self {
        Self {
            to: None,
            from: None,
            payload: payload.into(),
        }
    }

    pub fn to(id: u32, payload: impl Into<T>) -> Self {
        Self {
            to: Some(id),
            from: None,
            payload: payload.into(),
        }
    }

    pub fn from(id: u32, payload: impl Into<T>) -> Self {
        Self {
            to: None,
            from: Some(id),
            payload: payload.into(),
        }
    }
}
