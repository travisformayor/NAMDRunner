# NAMDRunner Documentation

This directory contains the complete NAMDRunner project documentation. Use this guide to find the information you need.

## 📋 Core Project Documentation

### [`project-spec.md`](project-spec.md)
**What we're building and why**
- Business requirements and goals
- End-to-end workflow description
- Success criteria and constraints
- Target platforms and deployment strategy

### [`technical-spec.md`](technical-spec.md) 
**Technology stack and development setup**
- Tauri v2 + Svelte + Rust architecture
- Development environment setup (Fedora VM)
- Testing strategy overview
- Coding standards and repository structure

## 🖥️ Alpine Cluster Integration

### [`cluster-guide.md`](cluster-guide.md) ⭐
**Complete Alpine cluster reference (SINGLE SOURCE OF TRUTH)**
- Hardware partitions and resource limits
- SLURM command patterns and job templates
- Module loading sequences for NAMD
- Error handling and recovery strategies
- Directory structure requirements

## 🔗 System Architecture

### [`api-spec.md`](api-spec.md)
**IPC interfaces and data contracts**
- TypeScript ↔ Rust command interfaces
- SSH/SFTP connection patterns
- Mock data for testing
- Error handling strategies

### [`data-spec.md`](data-spec.md)
**JSON schemas and database design**
- JSON metadata file formats
- SQLite schema definitions
- Data validation rules
- Schema versioning strategy

### [`architecture.md`](architecture.md)
**Current implementation state**
- What has been built (updated after each milestone)
- Component relationships and data flow
- Integration points and boundaries

### [`developer-guidelines.md`](developer-guidelines.md)
**Code quality standards and architectural patterns**
- Clean architecture principles from Phase 1 lessons
- Anti-patterns to avoid with concrete examples
- Result<T> error handling patterns
- Service development patterns
- Security and performance guidelines

## 🧪 Development Tools

### [`testing-spec.md`](testing-spec.md)
**Testing strategy and tools**
- Unit testing with Vitest (TypeScript) and Cargo (Rust)
- E2E testing with WebdriverIO + tauri-driver
- UI testing with Playwright
- Agent debugging toolkit

### [`agent-capabilities.md`](agent-capabilities.md)
**AI assistant development tools**
- Available testing commands and workflows
- Mock infrastructure for offline development
- Debug tools and utilities

## 📁 Reference Materials

### [`misc/`](misc/)
**Supplementary documentation**
- [`curc-docs-reference.md`](misc/curc-docs-reference.md) - Original CURC documentation dump (archived)
- [`design-discussion.md`](misc/design-discussion.md) - Design decisions and trade-offs
- [`tauri-docs-reference.md`](misc/tauri-docs-reference.md) - Tauri framework references

## 🗺️ Quick Navigation

**Starting a new task?**
1. Review [`project-spec.md`](project-spec.md) - understand the goals
2. Check [`cluster-guide.md`](cluster-guide.md) - get cluster integration details  
3. Reference [`api-spec.md`](api-spec.md) and [`data-spec.md`](data-spec.md) - understand data contracts
4. See [`testing-spec.md`](testing-spec.md) - set up testing workflow

**Need cluster information?**
- **Always use [`cluster-guide.md`](cluster-guide.md)** - the single authoritative source
- Contains partitions, QoS, resource limits, SLURM commands, job templates
- Replaces scattered cluster info previously in api-spec.md and data-spec.md

**Implementing features?**
1. Check [`../docs/reference/`](../docs/reference/) for Python implementation patterns
2. Use [`api-spec.md`](api-spec.md) for IPC interface definitions
3. Follow [`data-spec.md`](data-spec.md) for data schema requirements
4. Test with tools from [`testing-spec.md`](testing-spec.md)

**Need help with development environment?**
- See [`technical-spec.md`](technical-spec.md) for setup instructions
- Use [`agent-capabilities.md`](agent-capabilities.md) for debugging tools

## 📝 Documentation Maintenance

**When updating documentation:**
- **Cluster changes**: Update [`cluster-guide.md`](cluster-guide.md) only
- **Architecture changes**: Update [`architecture.md`](architecture.md) after implementation
- **New features**: Update relevant specs, add references to cluster-guide.md
- **Bug fixes**: Document lessons learned in appropriate reference files

**Documentation principles:**
- Single source of truth for cluster information
- Reference cluster-guide.md from other docs to avoid duplication
- Keep lessons learned separate from current facts
- Update architecture.md to reflect what IS built, not what WILL be built

---

*For questions about documentation organization or missing information, check the Git history or ask the development team.*