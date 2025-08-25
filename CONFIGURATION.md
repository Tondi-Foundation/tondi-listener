# Tondi Scan Configuration Guide

## 概述

Tondi Scan 项目使用统一的配置文件结构，所有配置都集中在工作区根目录的 `config.toml` 文件中。这种设计确保了配置的一致性和可维护性。

## 配置文件结构

```
tondi-scan/
├── config.toml          # 统一配置文件（主要配置）
├── config.example.toml  # 配置示例文件
├── env.example          # 环境变量示例
└── crates/
    ├── wasm2-client/    # WASM 客户端
    ├── server/          # 服务器端
    ├── http2-client/    # HTTP/2 客户端
    ├── http2-server/    # HTTP/2 服务器
    ├── http3-client/    # HTTP/3 客户端
    ├── wasm3-client/    # WASM3 客户端
    ├── library/         # 共享库
    └── db/              # 数据库模块
```

## 配置分区

### 1. 服务器配置 (`[server]`)
- 基本服务器设置（主机、端口、环境等）
- CORS 配置
- 安全设置
- 事件处理配置
- wRPC 配置

### 2. 客户端配置 (`[client]`)
- WASM 客户端连接设置
- 网络类型和编码类型
- 连接超时和重连设置
- 事件订阅配置

### 3. 数据库配置 (`[database]`)
- 数据库连接字符串
- 连接池设置
- 超时配置

### 4. 日志配置 (`[logging]`)
- 日志级别和格式
- 输出目标
- 文件路径设置

### 5. 监控配置 (`[monitoring]`)
- 指标收集
- 健康检查
- Prometheus 端点

## 端口映射规则

wRPC 端口根据网络类型和编码类型自动计算：

| 网络类型 | 编码类型 | 端口 |
|---------|---------|------|
| mainnet | borsh   | 17110 |
| mainnet | json    | 18110 |
| testnet | borsh   | 17210 |
| testnet | json    | 18210 |
| devnet  | borsh   | 17610 |
| devnet  | json    | 18610 |
| simnet  | borsh   | 17310 |
| simnet  | json    | 18310 |

## 环境变量支持

所有配置都支持通过环境变量覆盖：

```bash
# 服务器配置
export TONDI_SCAN_HOST_URL="0.0.0.0:3000"
export TONDI_SCAN_ENVIRONMENT="production"

# wRPC 配置
export TONDI_SCAN_WRPC_HOST="custom.host.com"
export TONDI_SCAN_WRPC_NETWORK="mainnet"
export TONDI_SCAN_WRPC_ENCODING="json"

# 数据库配置
export TONDI_SCAN_DATABASE_URL="postgres://user:pass@host/db"
```

## 配置优先级

1. 环境变量（最高优先级）
2. 配置文件中的值
3. 代码中的默认值（最低优先级）

## 使用示例

### 开发环境
```toml
[server]
environment = "development"
log_level = "debug"

[client]
log_level = "debug"
enable_console_log = true
```

### 生产环境
```toml
[server]
environment = "production"
log_level = "warn"

[client]
log_level = "info"
enable_console_log = false

[monitoring]
enabled = true
metrics_port = 9090
```

## 配置验证

所有配置在启动时都会进行验证：
- 端口范围检查（1024-65535）
- 网络类型验证
- 编码类型验证
- 协议类型验证
- 数据库连接测试

## 迁移指南

### 从旧配置迁移
1. 删除各模块的独立配置文件
2. 将配置项迁移到根目录的 `config.toml`
3. 更新代码以使用统一配置
4. 测试所有功能正常工作

### 配置项映射
| 旧配置位置 | 新配置位置 |
|-----------|-----------|
| `crates/wasm2-client/config.toml` | `config.toml` 的 `[client]` 部分 |
| `crates/server/src/ctx/config.rs` | `config.toml` 的 `[server]` 部分 |

## 最佳实践

1. **环境特定配置**：使用环境变量覆盖生产环境配置
2. **敏感信息**：不要在配置文件中硬编码密码或密钥
3. **配置版本控制**：将 `config.example.toml` 纳入版本控制
4. **配置验证**：在应用启动时验证所有配置项
5. **文档更新**：保持配置文档与代码同步

## 故障排除

### 常见问题

1. **配置文件未找到**
   - 确保 `config.toml` 在正确位置
   - 检查文件权限

2. **配置解析错误**
   - 验证 TOML 语法
   - 检查配置项类型

3. **端口冲突**
   - 验证端口号范围
   - 检查端口是否被占用

4. **环境变量未生效**
   - 确保环境变量名称正确
   - 重启应用以加载新环境变量

## 贡献指南

添加新配置项时：
1. 在 `config.toml` 中添加配置项
2. 在 `config.example.toml` 中添加示例
3. 更新相关文档
4. 添加配置验证逻辑
5. 编写测试用例
