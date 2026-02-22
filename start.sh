#!/bin/bash

# RustCloud 开发环境启动脚本
# Usage: ./start.sh [frontend|backend|all]

set -e

MODE=${1:-all}

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════╗"
    echo "║     RustCloud 开发环境启动器          ║"
    echo "╚═══════════════════════════════════════╝"
    echo -e "${NC}"
}

check_dependencies() {
    echo -e "${YELLOW}🔍 检查依赖...${NC}"
    
    # 检查 Rust
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Rust/Cargo 未安装${NC}"
        echo "请访问 https://rustup.rs 安装 Rust"
        exit 1
    fi
    
    # 检查 Node.js
    if ! command -v npm &> /dev/null; then
        echo -e "${RED}❌ Node.js/npm 未安装${NC}"
        echo "请访问 https://nodejs.org 安装 Node.js"
        exit 1
    fi
    
    echo -e "${GREEN}✅ 依赖检查通过${NC}"
}

install_deps() {
    echo -e "${YELLOW}📦 安装依赖...${NC}"
    
    if [ ! -d "backend/target" ]; then
        echo "安装后端依赖..."
        cd backend && cargo fetch && cd ..
    fi
    
    if [ ! -d "web/node_modules" ]; then
        echo "安装前端依赖..."
        cd web && npm install && cd ..
    fi
    
    echo -e "${GREEN}✅ 依赖安装完成${NC}"
}

start_backend() {
    echo -e "${BLUE}🚀 启动后端服务...${NC}"
    echo -e "   API: http://127.0.0.1:3000"
    echo -e "   文档: http://127.0.0.1:3000/swagger-ui"
    echo ""
    cd backend && cargo run
}

start_frontend() {
    echo -e "${BLUE}🚀 启动前端服务...${NC}"
    echo -e "   地址: http://localhost:5173"
    echo ""
    cd web && npm run dev
}

start_all() {
    echo -e "${YELLOW}🚀 同时启动前后端...${NC}"
    echo ""
    
    # 使用 trap 确保进程可以正确终止
    trap 'kill $(jobs -p) 2>/dev/null; exit' INT TERM EXIT
    
    # 后台启动后端
    start_backend &
    BACKEND_PID=$!
    
    # 等待后端启动
    sleep 3
    
    # 启动前端
    start_frontend &
    FRONTEND_PID=$!
    
    echo -e "${GREEN}"
    echo "✅ 所有服务已启动"
    echo ""
    echo "访问地址:"
    echo "  前端: http://localhost:5173"
    echo "  后端: http://127.0.0.1:3000"
    echo "  API 文档: http://127.0.0.1:3000/swagger-ui"
    echo ""
    echo "按 Ctrl+C 停止所有服务"
    echo -e "${NC}"
    
    # 等待所有进程
    wait
}

# 主逻辑
print_banner
check_dependencies
install_deps

case $MODE in
    backend)
        start_backend
        ;;
    frontend)
        start_frontend
        ;;
    all|*)
        start_all
        ;;
esac
