# Brightspace Plugin: Trinity Sandbox

**Status**: 💡 Idea (Not Started)
**Priority**: Future (After MVP)
**Target**: Purdue Global

---

## The Vision

Create a plugin that embeds Trinity's game sandbox (Iron Road) directly into Brightspace LMS, allowing:

1. **Teachers** to create constructivist learning scenarios
2. **Students** to explore physics-based educational games
3. **Social scaffolding** - Students help each other in-game

---

## Technical Approach

### Option A: LTI Integration

- Brightspace supports LTI (Learning Tools Interoperability)
- Trinity could be an external LTI tool
- Pro: Standard, works with many LMSs
- Con: Requires hosted server

### Option B: SCORM Package

- Package Iron Road as a SCORM module
- Pro: Runs entirely in browser (WASM)
- Con: Limited interactivity with LMS

### Option C: Brightspace API

- Direct integration via D2L's REST API
- Pro: Deep integration (grades, analytics)
- Con: Purdue-specific, more work

---

## Why This Matters (Educational Philosophy)

**Constructivism** says learners build knowledge through experience:

- Iron Road = The "doing" environment
- Brightspace = The "reflecting" environment
- Together = Complete learning cycle

---

## References

- [Brightspace LTI Docs](https://documentation.brightspace.com)
- [SCORM Standard](https://scorm.com/scorm-explained/)
- Iron Road Design Document: `/antigravity/Iron Road Design Document Creation.docx`

---

## Next Steps (When Ready)

1. [ ] Research LTI 1.3 requirements
2. [ ] Prototype WASM-in-iframe for Brightspace
3. [ ] Talk to Purdue Global about pilot program
