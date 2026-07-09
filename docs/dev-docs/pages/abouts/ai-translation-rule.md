<h1 align="center">AI Translation Rule</h1>
<p align="center">
    Translation prompt for your AI Agent
</p>

# Translation Style Guide

## 1. Tone & Voice

### Preserve original tone

Maintain the author's attitude, formality, and emotional register exactly as in the source.

### Synonymous substitution

Use words with close or equivalent meaning where direct translation is awkward or unnatural.

## 2. Vocabulary & Abbreviation

### Abbreviation

Apply standard English abbreviations (e.g., _info_ for information, _dept_ for department)
to avoid overlong words,
but only when clarity is not sacrificed.

### Concise expression

Prefer shorter, more common alternatives (e.g., _use_ over _utilize_, _help_ over _facilitate_)
unless the original tone demands formality.

## 3. Structural Rules

### Paragraph integrity

Keep the original paragraph breaks and line spacing.

### Tag preservation

Any inline Markdown formatting (bold, italic, code, links, lists) must be replicated exactly in translation.

### Example

- Before: “请保持专业语气，但避免使用过长的学术词汇。”
- After: “Keep a prof. tone, but avoid long academic words.”

### Minimal diff

When translating or syncing English content against a known Chinese original,
if the Chinese original's meaning is extremely close to the current English meaning,
do not modify the English text.
This is to keep git diffs friendly _(only modify parts that have truly changed)_.

## 4. Exceptions

- If a term has no common abbreviation, use the full word.
- If preserving tone requires a longer phrase, prioritize tone over brevity.

## 5. Original Text

```markdown
# Translation Style Guide

## 1. Tone & Voice

### Preserve original tone

Maintain the author's attitude, formality, and emotional register exactly as in the source.

### Synonymous substitution

Use words with close or equivalent meaning where direct translation is awkward or unnatural.

## 2. Vocabulary & Abbreviation

### Abbreviation

Apply standard English abbreviations (e.g., _info_ for information, _dept_ for department)
to avoid overlong words,
but only when clarity is not sacrificed.

### Concise expression

Prefer shorter, more common alternatives (e.g., _use_ over _utilize_, _help_ over _facilitate_)
unless the original tone demands formality.

## 3. Structural Rules

### Paragraph integrity

Keep the original paragraph breaks and line spacing.

### Tag preservation

Any inline Markdown formatting (bold, italic, code, links, lists) must be replicated exactly in translation.

### Example

- Before: “请保持专业语气，但避免使用过长的学术词汇。”
- After: “Keep a prof. tone, but avoid long academic words.”

### Minimal diff

When translating or syncing English content against a known Chinese original,
if the Chinese original's meaning is extremely close to the current English meaning,
do not modify the English text.
This is to keep git diffs friendly _(only modify parts that have truly changed)_.

## 4. Exceptions

- If a term has no common abbreviation, use the full word.
- If preserving tone requires a longer phrase, prioritize tone over brevity.
```

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
