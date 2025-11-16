# gh-proxy-rs

来源 [gh-proxy](https://github.com/hunshcn/gh-proxy) 的 Rust 语言版本重写，支持多 Git 服务代理。

## 功能特性

- 支持 GitHub、GitLab 和 Bitbucket 的代理
- 内置缓存机制，减少重复请求
- 请求速率限制，防止滥用
- 可配置的 jsDelivr 集成
- 灵活的配置系统（文件配置 + 环境变量）

## 支持的 Git 服务

### GitHub
- Releases 和归档文件
- Blob 和原始文件
- Gists
- Git 信息和标签

### GitLab
- 项目归档
- 原始文件和 blob

### Bitbucket
- 仓库归档
- 原始文件

## 配置

```toml
[server]
address = "127.0.0.1:4000"

[jsdelivr]
enabled = false

[cache]
enabled = true
max_capacity = 1000
time_to_live = 3600  # 1 hour

[rate_limit]
enabled = true
requests_per_minute = 60

[git_services]
gitlab_enabled = true
bitbucket_enabled = true
```

## 部署详情

[github.moeyy.xyz](https://github.moeyy.xyz/) 正在使用 **gh-proxy-go**，托管在 [BuyVM](https://buyvm.net/) 每月 3.5 美元的 1 核 1G 内存、10Gbps 带宽服务器上。

### 服务器概况：

- **日流量处理**：约 3TB
- **CPU 平均使用率**：20%
- **带宽平均占用**：400Mbps

![服务器数据](https://github.com/user-attachments/assets/6fe37f41-aa35-4efc-b0b8-8c3339529326)
![Cloudflare 数据](https://github.com/user-attachments/assets/ae310b1f-96e9-42e9-a77c-0d8c1b8d6344)

---

如有问题或改进建议，欢迎提交 issue 或 PR！
