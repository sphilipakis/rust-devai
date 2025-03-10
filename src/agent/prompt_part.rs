use genai::chat::ChatRole;

#[derive(Debug, Clone)]
pub struct PromptPart {
	pub kind: PartKind,
	pub content: String,
	pub options: Option<PartOptions>,
}

#[derive(Debug, Clone)]
pub enum PartKind {
	Instruction,
	System,
	Assistant,
}

#[derive(Debug, Clone, Default)]
pub struct PartOptions {
	pub cache: bool,
}

// region:    --- Froms

impl From<PartKind> for ChatRole {
	fn from(kind: PartKind) -> Self {
		match kind {
			PartKind::Instruction => ChatRole::User,
			PartKind::System => ChatRole::System,
			PartKind::Assistant => ChatRole::Assistant,
		}
	}
}

impl From<&PartKind> for ChatRole {
	fn from(kind: &PartKind) -> Self {
		match kind {
			PartKind::Instruction => ChatRole::User,
			PartKind::System => ChatRole::System,
			PartKind::Assistant => ChatRole::Assistant,
		}
	}
}

// endregion: --- Froms
