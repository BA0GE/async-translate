# 贡献指南

感谢您对 async-translate 项目的兴趣！我们欢迎各种形式的贡献，包括但不限于：

- 🐛 报告 bug
- 💡 提出新功能建议
- 📝 改进文档
- 🔧 提交代码修复
- ✨ 添加新功能

## 开发环境设置

### 要求

- Rust 1.70 或更高版本
- Cargo

### 克隆项目

```bash
git clone https://github.com/ba0ge/async-translate.git
cd async-translate
```

### 运行测试

```bash
cargo test
```

### 代码格式化和检查

```bash
# 格式化代码
cargo fmt

# 运行 Clippy 检查
cargo clippy

# 生成文档
cargo doc --open
```

## 开发流程

1. Fork 本仓库
2. 创建您的特性分支：`git checkout -b feature/amazing-feature`
3. 提交您的更改：`git commit -m 'Add some amazing feature'`
4. 推送到分支：`git push origin feature/amazing-feature`
5. 创建 Pull Request

## 代码规范

### Rust 代码规范

- 使用 `rustfmt` 格式化所有代码
- 遵循 Clippy 的建议
- 为公共 API 编写文档注释
- 添加适当的单元测试
- 使用有意义的变量和函数名

### 提交信息规范

请使用清晰简洁的提交信息：

```
类型：简短描述

详细说明（如果需要）
```

类型包括：
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式化
- `refactor`: 代码重构
- `test`: 添加测试
- `chore`: 构建工具或辅助工具的变动

### 测试要求

- 为新功能添加单元测试
- 确保所有现有测试通过
- 保持良好的测试覆盖率

## 报告问题

如果您发现 bug，请：

1. 检查是否已存在相关 issue
2. 如果没有，创建一个新的 issue
3. 提供详细的复现步骤
4. 包含相关环境信息

## 许可证

通过贡献代码，您同意您的贡献将采用与项目相同的 MIT 许可证。