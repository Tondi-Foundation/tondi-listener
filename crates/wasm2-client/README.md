# Tondi Listener WASM Client

## 配置说明

### 1. 配置原则
- **不再硬编码 URL**：所有连接信息都通过配置文件或参数提供
- **自动端口计算**：根据网络类型和编码类型自动计算端口
- **配置一致性**：与服务器端配置结构保持一致
- **统一配置文件**：使用工作区根目录的 `config.toml` 文件

### 2. 端口映射规则
```
mainnet + borsh = 17110
mainnet + json  = 18110
testnet + borsh = 17210
testnet + json  = 18210
devnet + borsh  = 17610
devnet + json   = 18610
simnet + borsh  = 17310
simnet + json   = 18310
```

### 3. 配置选项

#### 基本连接配置
- `url`: 完整的连接 URL（可选，如果不提供将自动构建）
- `host`: 主机地址
- `protocol`: 协议类型（ws/wss）
- `network_id`: 网络类型（mainnet/testnet/devnet/simnet）
- `encoding`: 编码类型（borsh/json）

#### 连接参数
- `connection_timeout_ms`: 连接超时时间（毫秒）
- `ping_interval_ms`: Ping 间隔（毫秒）
- `auto_reconnect`: 是否自动重连
- `max_reconnect_attempts`: 最大重连次数
- `reconnect_delay_ms`: 重连延迟（毫秒）

#### 事件配置
- `default_events`: 默认订阅的事件类型
- `log_level`: 日志级别
- `enable_console_log`: 是否启用控制台日志

### 4. 使用示例

#### 使用默认配置（自动计算端口）
```javascript
const config = {
    encoding: "borsh",
    network_id: "devnet"
    // 将自动构建 URL: wss://8.210.45.192:17610
};

const client = new TondiScanClient(config);
```

#### 使用自定义配置
```javascript
const config = {
    url: "wss://custom.host:8080",
    encoding: "json",
    network_id: "mainnet"
};

const client = new TondiScanClient(config);
```

#### 使用统一配置文件
项目使用工作区根目录的 `config.toml` 文件，包含所有配置：

```toml
[client]
default_network = "devnet"
default_encoding = "borsh"
default_host = "8.210.45.192"
default_protocol = "wss"
connection_timeout_ms = 10000
ping_interval_ms = 30000
auto_reconnect = true
max_reconnect_attempts = 5
reconnect_delay_ms = 1000
```

### 5. 环境变量支持
支持通过环境变量覆盖配置：
- `TONDI_SCAN_WRPC_HOST`: wRPC 主机地址
- `TONDI_SCAN_WRPC_NETWORK`: 网络类型
- `TONDI_SCAN_WRPC_ENCODING`: 编码类型
- `TONDI_SCAN_WRPC_PROTOCOL`: 协议类型

### 6. 配置验证
- 自动验证网络类型和编码类型
- 端口范围验证（1024-65535）
- 协议类型验证（ws/wss）

### 7. 注意事项
- 如果未提供 URL，将根据网络类型和编码类型自动计算端口
- 配置与服务器端保持一致，避免配置不一致问题
- 支持环境变量覆盖，便于部署时配置
- 所有硬编码都已移除，配置完全可定制
- 使用统一的 `config.toml` 文件，便于管理和维护

### 8. 配置文件结构
```
tondi-listener/
├── config.toml          # 统一配置文件
├── config.example.toml  # 配置示例文件
├── env.example          # 环境变量示例
└── crates/
    ├── wasm2-client/    # WASM 客户端
    ├── server/          # 服务器端
    └── ...
```

所有配置都集中在根目录的 `config.toml` 文件中，确保配置的一致性和可维护性。
