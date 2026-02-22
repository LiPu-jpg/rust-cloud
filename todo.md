# RustCloud 开发计划

## 功能实现状态总览

### ✅ 已完成功能
| 功能 | 服务端 | 前端 | CLI |
|------|--------|------|-----|
| 文件 CRUD | ✅ | ✅ | ✅ |
| 创建文件夹 | ✅ | ✅ | ✅ |
| 设备管理 | ✅ | ✅ | - |
| 版本历史 | ✅ | ✅ | - |
| 同步计划/执行 | ✅ | ✅ | ✅ |
| 文件监控 | ✅ | - | - |
| 哈希去重存储 | ✅ | - | - |

---

## CLI 客户端 (rcloud)

### 已实现命令
```bash
# 配置
rcloud config --server http://127.0.0.1:3000 --device-name my-laptop

# 同步
rcloud sync                    # 同步当前目录
rcloud sync --path ~/Documents # 同步指定目录
rcloud sync --dry-run          # 预览同步操作

# 状态
rcloud status                  # 查看同步状态

# 文件操作
rcloud ls                      # 列出远程文件
rcloud ls --path /docs         # 列出指定目录
rcloud upload local.txt        # 上传文件
rcloud upload local.txt --remote-path docs/remote.txt
rcloud download remote.txt     # 下载文件
```

### 待完善
- [ ] 本地文件监控 (实时同步)
- [ ] 增量同步 (只传输变化部分)
- [ ] 断点续传
- [ ] 冲突检测与解决
- [ ] 本地状态数据库

---

## 一键启动

```bash
make dev        # 同时启动前后端
make backend    # 后端: http://127.0.0.1:3000
make frontend   # 前端: http://localhost:5173
make test       # 运行测试

# CLI 使用
./target/release/rcloud --help
```

---

最后更新: 2026-02-23
状态: CLI 客户端基础框架完成
