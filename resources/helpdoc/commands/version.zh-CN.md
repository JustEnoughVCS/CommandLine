> 显示 `jvn` 工具的版本信息

## 使用
jvn version              # 显示基础版本号
jvn version --no-banner  # 不显示横幅，仅输出版本信息

jvn version --with-compile-info
jvn version -c           # 显示版本号及编译信息

## 别名
您可以使用 `-v` 或 `--version` 符号将命令重定向到 version

### 例如
jvn -v -c
jvn --version --no-banner
