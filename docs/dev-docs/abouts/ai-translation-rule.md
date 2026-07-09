# Translation Style Guide

## 1. Tone & Voice

- **保持原语气** (Preserve original tone): Maintain the author's attitude, formality, and emotional register exactly as in the source.
- **近似词替换** (Synonymous substitution): Use words with close or equivalent meaning where direct translation is awkward or unnatural.

## 2. Vocabulary & Abbreviation

- **缩写** (Abbreviation): Apply standard English abbreviations (e.g., _info_ for information, _dept_ for department) to avoid overlong words, but only when clarity is not sacrificed.
- **简明表述** (Concise expression): Prefer shorter, more common alternatives (e.g., _use_ over _utilize_, _help_ over _facilitate_) unless the original tone demands formality.

## 3. Structural Rules

- **段落一致** (Paragraph integrity): Keep the original paragraph breaks and line spacing.
- **标记保留** (Tag preservation): Any inline Markdown formatting (bold, italic, code, links, lists) must be replicated exactly in translation.
- **例示** (Example):
  - 原句: “请保持专业语气，但避免使用过长的学术词汇。”
  - 译文: “Keep a prof. tone, but avoid long academic words.”
- **最小化改动** (Minimal diff): When translating or syncing English content against a known Chinese original, if the Chinese original's meaning is extremely close to the current English meaning, do not modify the English text. This is to keep git diffs friendly (only modify parts that have truly changed).

## 4. Exceptions

- If a term has no common abbreviation, use the full word.
- If preserving tone requires a longer phrase, prioritize tone over brevity.
