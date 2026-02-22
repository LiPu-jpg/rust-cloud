.PHONY: help dev build test clean install frontend backend

# 默认显示帮助
help:
	@echo "RustCloud - 文件同步与存储服务"
	@echo ""
	@echo "可用命令:"
	@echo "  make install      - 安装前后端依赖"
	@echo "  make dev          - 启动开发环境（前后端同时）"
	@echo "  make backend      - 只启动后端服务"
	@echo "  make frontend     - 只启动前端开发服务器"
	@echo "  make build        - 构建前后端生产版本"
	@echo "  make test         - 运行所有测试"
	@echo "  make clean        - 清理构建产物"
	@echo "  make docker-up    - 使用 Docker 启动（如配置）"
	@echo ""

# 安装依赖
install:
	@echo "📦 安装后端依赖..."
	cd backend && cargo fetch
	@echo "📦 安装前端依赖..."
	cd web && npm install
	@echo "✅ 依赖安装完成"

# 开发环境（同时启动前后端）
dev:
	@echo "🚀 启动开发环境..."
	@echo "后端: http://127.0.0.1:3000"
	@echo "前端: http://localhost:5173"
	@echo "API 文档: http://127.0.0.1:3000/swagger-ui"
	@echo ""
	@make -j2 backend frontend

# 启动后端
backend:
	cd backend && cargo run

# 启动前端
frontend:
	cd web && npm run dev

# 构建生产版本
build:
	@echo "🔨 构建后端..."
	cd backend && cargo build --release
	@echo "🔨 构建前端..."
	cd web && npm run build
	@echo "✅ 构建完成"
	@echo "后端: ./backend/target/release/rustcloud"
	@echo "前端: ./web/dist/"

# 运行测试
test:
	@echo "🧪 运行后端测试..."
	cd backend && cargo test
	@echo "🧪 运行前端测试..."
	cd web && npm test || true

# 清理构建产物
clean:
	@echo "🧹 清理构建产物..."
	cd backend && cargo clean
	rm -rf web/dist
	rm -rf web/node_modules
	@echo "✅ 清理完成"

# 代码格式化
fmt:
	@echo "📝 格式化代码..."
	cd backend && cargo fmt
	cd web && npm run lint -- --fix 2>/dev/null || true

# 检查代码
lint:
	@echo "🔍 检查代码..."
	cd backend && cargo clippy -- -D warnings
	cd web && npm run lint

# Docker 启动（如配置了 docker-compose.yml）
docker-up:
	docker-compose up --build -d

docker-down:
	docker-compose down

# 创建必要目录
setup:
	mkdir -p backend/storage
	mkdir -p backend/logs
	@echo "✅ 目录创建完成"
