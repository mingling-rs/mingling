/// `OSC 9;4` 协议消息
///
/// 用于通过 ANSI 转义序列向终端发送任务进度通知消息
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OSC94State {
    /// 清除/隐藏进度（任务完成时使用），对应状态码 `0`
    Clean,
    /// 正常状态，对应状态码 `1`，需要配合进度值（0-100）
    Normal(f32),
    /// 错误状态，对应状态码 `2`（通常显示为红色）
    Error,
    /// 不确定状态，对应状态码 `3`（显示为无限循环的动画，用于进度未知的任务）
    Unknown,
    /// 警告状态，对应状态码 `4`（通常显示为黄色）
    Warn,
}

impl OSC94State {
    /// Returns the state code for the `OSC 9;4` protocol.
    #[must_use]
    pub const fn state_code(&self) -> u8 {
        match self {
            Self::Clean => 0,
            Self::Normal(_) => 1,
            Self::Error => 2,
            Self::Unknown => 3,
            Self::Warn => 4,
        }
    }

    /// Returns the progress value (0-100) for the `Normal` state, clamped to the valid range.
    #[must_use]
    pub const fn progress(&self) -> f32 {
        match self {
            Self::Normal(progress) => (progress.clamp(0.0, 1.0) * 100.0).round(),
            _ => 0.0,
        }
    }

    /// Converts the message into the corresponding `OSC 9;4` escape sequence string.
    #[must_use]
    pub fn to_escape_sequence(&self) -> String {
        format!("\x1b]9;4;{};{}\x07", self.state_code(), self.progress())
    }

    /// Sends the OSC 9;4 message to the terminal via stdout.
    ///
    /// # Panics
    ///
    /// Panics if the stdout stream cannot be flushed.
    pub fn send(&self) {
        use std::io::Write;
        print!("{}", self.to_escape_sequence());
        std::io::stdout().flush().unwrap();
    }
}

impl std::fmt::Display for OSC94State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_escape_sequence())
    }
}

impl From<OSC94State> for String {
    fn from(msg: OSC94State) -> Self {
        msg.to_escape_sequence()
    }
}

impl From<&OSC94State> for String {
    fn from(msg: &OSC94State) -> Self {
        msg.to_escape_sequence()
    }
}
