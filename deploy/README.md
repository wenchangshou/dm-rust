# Kubernetes 部署

## 文件说明

### MySQL 数据库
| 文件 | 说明 |
|------|------|
| `mysql-secret.yaml` | MySQL 密码配置 |
| `mysql-configmap.yaml` | 初始化 SQL 脚本 |
| `mysql-pvc.yaml` | MySQL 持久卷声明（10Gi） |
| `mysql-deployment.yaml` | MySQL 5.5.44 Deployment |
| `mysql-service.yaml` | MySQL Service |

### dm-rust 应用
| 文件 | 说明 |
|------|------|
| `dm-rust-configmap.yaml` | 应用配置文件 |
| `dm-rust-pvc.yaml` | 数据持久卷声明（5Gi） |
| `dm-rust-deployment.yaml` | 应用 Deployment |
| `dm-rust-service.yaml` | 应用 Service（ClusterIP + NodePort） |

### 其他
| 文件 | 说明 |
|------|------|
| `kustomization.yaml` | Kustomize 配置 |

---

## 构建 Docker 镜像

```bash
# 在项目根目录执行
cd /path/to/dm-rust

# 构建镜像
docker build -t dm-rust:latest .

# 如果使用私有仓库
docker tag dm-rust:latest your-registry/dm-rust:latest
docker push your-registry/dm-rust:latest
```

---

## 部署步骤

### 1. 修改配置

#### 修改 MySQL 密码
编辑 `mysql-secret.yaml`：
```yaml
stringData:
  mysql-root-password: "你的root密码"
  mysql-password: "你的用户密码"
```

同步更新 `mysql-configmap.yaml` 和 `dm-rust-configmap.yaml` 中的密码。

#### 修改应用配置
编辑 `dm-rust-configmap.yaml`，根据需要添加设备通道、节点等配置。

### 2. 部署

```bash
# 使用 kustomize 一键部署（推荐）
kubectl apply -k deploy/

# 或者按顺序部署
kubectl apply -f deploy/mysql-secret.yaml
kubectl apply -f deploy/mysql-configmap.yaml
kubectl apply -f deploy/mysql-pvc.yaml
kubectl apply -f deploy/mysql-deployment.yaml
kubectl apply -f deploy/mysql-service.yaml
kubectl apply -f deploy/dm-rust-configmap.yaml
kubectl apply -f deploy/dm-rust-pvc.yaml
kubectl apply -f deploy/dm-rust-deployment.yaml
kubectl apply -f deploy/dm-rust-service.yaml
```

### 3. 验证部署

```bash
# 查看所有 Pod
kubectl get pods -l project=dm-rust

# 查看服务
kubectl get svc -l project=dm-rust

# 查看 PVC
kubectl get pvc

# 查看 dm-rust 日志
kubectl logs -l app=dm-rust -f

# 查看 MySQL 日志
kubectl logs -l app=mysql -f
```

---

## 访问服务

### 集群内部访问
```
http://dm-rust:18080
```

### 集群外部访问（NodePort）
```
http://<节点IP>:31080
```

### API 示例
```bash
# 健康检查
curl http://<节点IP>:31080/

# 获取所有 Screen
curl http://<节点IP>:31080/api/screens

# 创建 Screen
curl -X POST http://<节点IP>:31080/api/screens \
  -H "Content-Type: application/json" \
  -d '{"type":"Normal","name":"测试","content":"内容","active":true}'
```

---

## 配置更新

修改 ConfigMap 后需要重启 Pod 生效：

```bash
# 更新配置
kubectl apply -f deploy/dm-rust-configmap.yaml

# 重启应用
kubectl rollout restart deployment dm-rust
```

---

## 清理

```bash
# 删除所有资源
kubectl delete -k deploy/

# 或者分别删除
kubectl delete -f deploy/dm-rust-service.yaml
kubectl delete -f deploy/dm-rust-deployment.yaml
kubectl delete -f deploy/dm-rust-pvc.yaml
kubectl delete -f deploy/dm-rust-configmap.yaml
kubectl delete -f deploy/mysql-service.yaml
kubectl delete -f deploy/mysql-deployment.yaml
kubectl delete -f deploy/mysql-pvc.yaml
kubectl delete -f deploy/mysql-configmap.yaml
kubectl delete -f deploy/mysql-secret.yaml
```

**⚠️ 注意**: 删除 PVC 会同时删除持久卷中的数据！

---

## 故障排查

```bash
# 查看 Pod 事件
kubectl describe pod <pod-name>

# 进入容器
kubectl exec -it <pod-name> -- /bin/sh

# 查看 MySQL 连接
kubectl exec -it $(kubectl get pod -l app=mysql -o jsonpath="{.items[0].metadata.name}") -- mysql -uroot -p

# 测试网络连通性
kubectl run -it --rm debug --image=busybox -- nc -zv mysql 3306
```
