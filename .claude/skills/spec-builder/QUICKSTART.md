# 🚀 Quick Start Guide: Spec Builder Skill

## ✅ Installation Complete!

Chúc mừng! Bạn đã cài đặt thành công **Spec Builder Skill** cho Claude Code.

---

## 📦 What's Installed?

```
.claude/skills/spec-builder/
├── SKILL.md          # Core skill instructions (14KB)
├── README.md         # Full documentation (9KB)
├── EXAMPLE.md        # Complete working example (22KB)
├── TEST_CASES.md     # 10 test scenarios (11KB)
└── QUICKSTART.md     # This file
```

---

## 🎯 What Does This Skill Do?

**Spec Builder** tự động tạo tài liệu đầy đủ từ yêu cầu của bạn:

1. **requirements.md** → User stories + Acceptance criteria (EARS format)
2. **design.md** → Kiến trúc kỹ thuật + Component design + API contracts
3. **tasks.md** → Task breakdown chi tiết với dependencies

**Perfect cho:**
- Product Managers: Viết requirements rõ ràng
- Developers: Hiểu hệ thống và implementation steps
- AI Coding Agents: Có tài liệu structured để implement

---

## 🏃 Try It Now! (3 phút)

### Bước 1: Mở Claude Code trong workspace này

Đảm bảo bạn đang ở trong:
```
/Users/ngocp/Documents/projects/clean-macos
```

### Bước 2: Thử lệnh đơn giản

Copy và paste vào Claude:

```
Use spec-builder to generate a full spec for:
"Add a dark mode toggle button to the settings page with theme persistence"
```

### Bước 3: Quan sát kết quả

Claude sẽ:
- ✅ Nhận diện skill spec-builder
- ✅ Tạo requirements.md với user stories
- ✅ Tạo design.md với technical architecture
- ✅ Tạo tasks.md với implementation steps

### Bước 4: Review output

Kiểm tra 3 files được tạo:
- Có user stories đúng format không?
- Design có đầy đủ components không?
- Tasks có thể implement được không?

---

## 💡 More Examples

### Example 1: Backend Feature
```
spec-builder: Generate specs for a rate limiting API middleware 
that prevents abuse while allowing burst traffic. Support Redis for 
distributed systems.
```

### Example 2: Complex Feature
```
I need comprehensive specs using spec-builder for:

"Real-time collaborative document editor with operational transformation,
presence indicators, cursor tracking, and conflict resolution"
```

### Example 3: Lightweight Spec
```
Create a minimal spec with spec-builder for:
"Add loading spinner to dashboard during data fetch"
```

---

## 📊 Skill Info

| Property | Value |
|----------|-------|
| **Name** | `spec-builder` |
| **Type** | Project Skill (shared via git) |
| **Allowed Tools** | Write, Read |
| **Auto-activated by** | Keywords: "spec", "requirements", "design doc", "task breakdown" |
| **Output Format** | 3 Markdown files |

---

## 🧪 Run Tests

Xem [TEST_CASES.md](./TEST_CASES.md) để có 10 test scenarios chi tiết.

**Quick test:**
```
Use spec-builder for: "Add export to PDF button"
```

Expected: 3 complete documents in ~2-3 minutes.

---

## 📖 Learn More

| Document | Purpose | Read Time |
|----------|---------|-----------|
| [README.md](./README.md) | Full documentation | 10 min |
| [SKILL.md](./SKILL.md) | Skill instructions & templates | 15 min |
| [EXAMPLE.md](./EXAMPLE.md) | Complete example: Auth with 2FA | 20 min |
| [TEST_CASES.md](./TEST_CASES.md) | 10 test scenarios | 15 min |

---

## ✨ Tips for Best Results

### ✅ DO:
- **Be specific**: "Add user profile page with avatar upload" 
- **Include context**: "For our e-commerce React app..."
- **Mention constraints**: "Must work offline, < 100ms response"
- **Name users**: "As an admin user..." vs "As a user..."

### ❌ DON'T:
- **Be vague**: "Make the app better"
- **Skip context**: "Add authentication" (what kind?)
- **Assume details**: Provide tech stack, requirements, constraints

---

## 🎨 Customization

### Request Different Detail Levels

**Minimal:**
```
Generate lightweight spec for: [feature]
```

**Detailed:**
```
I need comprehensive, production-ready specs with code examples for: [feature]
```

### Focus on Specific Sections

**Only design:**
```
I have requirements already. Use spec-builder to generate just the design doc for: [feature]
```

**Only tasks:**
```
Generate task breakdown from this design doc: [attach design.md]
```

---

## 🔧 Troubleshooting

### Skill không activate?

**Solution 1:** Restart Claude Code
```bash
# Restart Claude Code application
```

**Solution 2:** Explicitly mention skill name
```
Use spec-builder to create specs for: [your feature]
```

**Solution 3:** Check installation
```bash
ls -la .claude/skills/spec-builder/SKILL.md
```

### Output quá ngắn?

```
I need more detailed technical specs with code examples for: [feature]
```

### Output quá dài?

```
Generate concise spec for: [feature]
```

---

## 🤝 Share with Team

Skill này đã được cài đặt trong project, sẽ tự động share qua git:

```bash
# Team members chỉ cần pull latest code
git pull

# Skill sẽ tự động available
```

Không cần setup gì thêm! ✅

---

## 📈 Success Metrics

Một spec tốt phải:

- ✅ **Complete**: Requirements → Design → Tasks
- ✅ **Clear**: Anyone can understand
- ✅ **Actionable**: Tasks ready to implement
- ✅ **Testable**: Acceptance criteria measurable
- ✅ **Maintainable**: Easy to update

---

## 🎯 Real-World Use Cases

### Use Case 1: Planning Sprint
```
PM: "Generate specs for user authentication feature"
→ Requirements reviewed by stakeholders
→ Design approved by architects
→ Tasks distributed to developers
```

### Use Case 2: AI Agent Development
```
Developer: "Create implementation plan for caching layer"
→ Feed tasks.md to Cursor/Copilot
→ AI implements each task sequentially
→ Faster development with clear guidance
```

### Use Case 3: Documentation
```
Architect: "Document our microservices architecture"
→ Generate design docs for each service
→ Maintain up-to-date technical documentation
→ Onboard new team members faster
```

---

## 🆘 Need Help?

### In-Skill Help
Ask Claude:
```
What can spec-builder do?
Show me spec-builder examples
How do I use spec-builder for API design?
```

### Documentation
- **Quick reference**: This file (QUICKSTART.md)
- **Full guide**: [README.md](./README.md)
- **Templates**: [SKILL.md](./SKILL.md)
- **Examples**: [EXAMPLE.md](./EXAMPLE.md)

### Common Questions

**Q: Can this replace product managers?**
A: No, it helps PMs work more efficiently by automating documentation.

**Q: Does this work with Cursor/Copilot?**
A: Yes! The task breakdown format is optimized for AI coding agents.

**Q: Can I customize the output format?**
A: Yes, just specify: "Use our company's template format" and provide your template.

---

## 📊 Task Progress Tracking (New!)

After generating `tasks.md`, track your implementation progress:

```bash
# Extract tasks to CSV
cd .claude/skills/spec-builder/scripts
python track_progress.py extract ../../specs/[feature-name]/tasks.md

# View progress
python track_progress.py progress

# Get next task to implement
python track_progress.py next

# Mark task as done
python track_progress.py done <task_id>
```

See SKILL.md for full tracking documentation.

---

## 🎉 Next Steps

1. ✅ **Try the quick example above** (3 minutes)
2. ✅ **Read EXAMPLE.md** for a complete Auth 2FA spec (20 minutes)
3. ✅ **Generate specs for your current feature** (actual work!)
4. ✅ **Track implementation progress** (use tracking system)
5. ✅ **Share with your team** (git commit + push)
6. ✅ **Iterate and improve** (refine based on feedback)

---

## 🌟 Feedback

Found this useful? Have suggestions?

- Update SKILL.md with improvements
- Add your examples to EXAMPLE.md
- Share test results in TEST_CASES.md
- Commit improvements and share with team

---

## 📝 Summary

| Status | Item |
|--------|------|
| ✅ | Skill installed |
| ✅ | Documentation complete |
| ✅ | Examples provided |
| ✅ | Tests available |
| 🎯 | **Ready to use!** |

**Try it now:**
```
Use spec-builder to generate a full spec for:
"[Your feature idea here]"
```

---

**Happy spec building!** 🚀

Created: 2025-12-20
Updated: 2025-01-XX
Version: 1.1.0 (with task tracking)
Project: clean-macos

