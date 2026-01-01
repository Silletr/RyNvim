# 🚀 RyNvim Quest Log

Priority: 1st = 🔥 ASAP | 2nd = ⚡ Soon | 3rd = 📅 Later

**01/01/2026** 🎮 Basic Functional Tier:

1. ⌨️ **Keyboard Parser** [PL-1st] 🔥
   - Parse key events (raw input → tokens) [~2h]
   - Bind via PyO3: `src/backend/keyboard/parser.py` ↔ `src/core/keyboard.rs` [~1h]
   - Test: Echo keys to statusline [~30m]
   - ✅ Progress: [ ] 0%

2. 📝 **File Text Writer** [PL-1st] 🔥
   - Prompt user for filename (By python, from keyboard module)
   - Append/insert text from buffer to file [~1h]
   - Files: `src/core/buffer.rs` + `runtime/lua/io.lua` [~1h]
   - Edge cases: Overwrite? Newline handling [~30m]
   - ✅ Progress: [ ] 0%
   - Depends: Keyboard parser done

**Daily Win Tracker:**

- [ ] Today's focus: Keyboard parser
- Est. total: 6h | Burn rate: ⏳
