> 工作区管理命令

## 使用
jvn workspace <子命令> <参数: ?>

## 子命令

### 初始化工作区
在_当前目录_初始化一个 `工作区`
jvn workspace init

### 创建工作区
在_指定目录_初始化一个 `工作区`
jvn workspace create <目录>

### 操作结构表
操作或显示_当前工作区_下的 `结构表`
jvn workspace sheet <参数: ?>

> 您可以使用如下查询详细用例
> `jvn helpdoc commands/workspace/sheet`

### 操作 ID 别名
操作或读取_当前工作区_下的 `ID 别名映射`
jvn workspace alias <参数: ?>

> 您可以使用如下查询详细用例
> `jvn helpdoc commands/workspace/alias`

### 查询工作区所在目录
打印_当前工作区_的目录
jvn workspace here
