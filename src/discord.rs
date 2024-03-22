struct Discord {
    token: String,
    data: types::Data,
}

impl Discord {
    pub fn New(token: String, data: types::Data) -> Self {
        Self { token, data }
    }

    pub fn Init(&self) ->  {
        // Initialize Discord
    }
}
