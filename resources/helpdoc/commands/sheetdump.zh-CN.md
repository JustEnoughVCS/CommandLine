> 可视化地输出 `Sheet` 的内部结构

## 使用
jvn sheetdump <SHEET_FILE>              # 默认输出
jvn sheetdump <SHEET_FILE> --no-sort    # 无排序
jvn sheetdump <SHEET_FILE> --no-pretty  # 无美化

## 提示
您也可以使用 `渲染器重载` 来访问 `Sheet` 的内部结构，例如
jvn sheetdump <SHEET_FILE> --renderer json | jq ".mappings"
