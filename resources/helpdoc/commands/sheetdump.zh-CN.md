> 可视化地输出 `结构表` 的内部结构

## 使用
jvn sheetdump <结构表文件>              # 默认输出
jvn sheetdump <结构表文件> --no-sort    # 无排序
jvn sheetdump <结构表文件> --no-pretty  # 无美化

## 提示
您也可以使用 `渲染器重载` 来访问 `结构表` 的内部结构，例如
jvn sheetdump <结构表文件> --renderer json | jq ".mappings"
