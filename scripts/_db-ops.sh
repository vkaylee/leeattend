#!/bin/bash

# Guard: Prevent direct execution. Must be sourced or run via official entry point.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo -e "\033[0;31m❌ SECURITY VIOLATION: Direct execution of this module is forbidden.\033[0m"
    echo -e "👉 This script is an internal module. Use: \033[1mleedevkit test\033[0m"
    exit 1
fi


# Guard: Prevent direct execution. Must be sourced or run via official entry point.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo -e "\033[0;31m❌ SECURITY VIOLATION: Direct execution of this module is forbidden.\033[0m"
    echo -e "👉 This script is an internal module. Use: \033[1mleedevkit test\033[0m"
    exit 1
fi


# --- Database backup/restore functions ---
_COMPOSE_ENGINE="${DOCKER_COMPOSE_CMD:-podman-compose}"
_DEV_COMPOSE_CMD="$_COMPOSE_ENGINE -p leeattend-dev -f $PROJECT_ROOT/docker-compose.yml -f $PROJECT_ROOT/.compose/docker-compose.dev.yml"

db_backup() {
    local backup_dir="$SCRIPT_DIR/backups"
    local timestamp
    timestamp=$(date +%Y%m%d_%H%M%S)
    local filename="leeattend_${timestamp}.dump"
    local filepath="$backup_dir/$filename"

    # Load .env for DB credentials
    set -a
    # shellcheck disable=SC1091
    [ -f "$SCRIPT_DIR/.env" ] && source "$SCRIPT_DIR/.env"
    set +a
    local db_user="${DB_USER:-postgres}"
    local db_name="${DB_NAME:-leeattend}"
    local db_password="${DB_PASSWORD:-postgres}"
    local pooler_port="${PGBOUNCER_PORT:-6432}"

    echo -e "${BLUE}${BOLD}🗄️ PostgreSQL Backup${NC}"
    echo -e "${BLUE}  Database: ${db_name}  User: ${db_user}${NC}"

    # Check that db container is running and ready
    if ! PGPASSWORD="$db_password" $_DEV_COMPOSE_CMD exec -T -e PGPASSWORD db \
        pg_isready -h pgbouncer -p "$pooler_port" -U "$db_user" -d "$db_name" \
        </dev/null >/dev/null 2>&1; then
        echo -e "${RED}❌ Database container is not running or not ready.${NC}"
        echo -e "${YELLOW}Start it first: leedevkit manage up dev${NC}"
        exit 1
    fi

    mkdir -p "$backup_dir"

    echo -e "${BLUE}  Dumping to: ${YELLOW}./backups/$filename${NC}"
    if PGPASSWORD="$db_password" $_DEV_COMPOSE_CMD exec -T -e PGPASSWORD db \
        pg_dump -h pgbouncer -p "$pooler_port" -U "$db_user" -d "$db_name" -Fc --no-owner --no-acl \
        </dev/null > "$filepath" 2>/dev/null; then
        local size
        size=$(du -h "$filepath" | cut -f1)
        echo -e "${GREEN}✅ Backup complete: ./backups/$filename ($size)${NC}"
    else
        rm -f "$filepath"
        echo -e "${RED}❌ Backup failed!${NC}"
        exit 1
    fi
}

db_restore() {
    local file="$1"

    # Resolve path: bare filename → look in ./backups/, relative → resolve from project root
    if [[ "$file" != /* ]]; then
        if [[ "$file" != */* && -f "$SCRIPT_DIR/backups/$file" ]]; then
            file="$SCRIPT_DIR/backups/$file"
        else
            file="$SCRIPT_DIR/$file"
        fi
    fi

    if [[ ! -f "$file" ]]; then
        echo -e "${RED}❌ File not found: $file${NC}"
        echo -e "${YELLOW}Available backups:${NC}"
        ls -1 "$SCRIPT_DIR/backups/"*.dump "$SCRIPT_DIR/backups/"*.sql 2>/dev/null || echo "  (none)"
        exit 1
    fi

    # Load .env for DB credentials
    set -a
    # shellcheck disable=SC1091
    [ -f "$SCRIPT_DIR/.env" ] && source "$SCRIPT_DIR/.env"
    set +a
    local db_user="${DB_USER:-postgres}"
    local db_name="${DB_NAME:-leeattend}"
    local db_password="${DB_PASSWORD:-postgres}"
    local pooler_port="${PGBOUNCER_PORT:-6432}"

    echo -e "${BLUE}${BOLD}🗄️ PostgreSQL Restore${NC}"
    echo -e "${BLUE}  Database: ${db_name}  From: ${file}${NC}"

    # Safety prompt (skip in non-interactive / CI)
    if [[ -t 0 && -z "${CI:-}" ]]; then
        echo -e "${YELLOW}⚠️  WARNING: This will OVERWRITE all data in '${db_name}'.${NC}"
        read -t 10 -r -p "Continue? [y/N] " confirm || confirm="N"
        if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
            echo "Aborted."
            exit 0
        fi
    fi

    # Check that db container is running and ready
    if ! PGPASSWORD="$db_password" $_DEV_COMPOSE_CMD exec -T -e PGPASSWORD db \
        pg_isready -h pgbouncer -p "$pooler_port" -U "$db_user" -d "$db_name" \
        </dev/null >/dev/null 2>&1; then
        echo -e "${RED}❌ Database container is not running or not ready.${NC}"
        echo -e "${YELLOW}Start it first: leedevkit manage up dev${NC}"
        exit 1
    fi

    local ext="${file##*.}"
    case "$ext" in
        dump)
            echo -e "${BLUE}  Restoring (pg_restore --clean)...${NC}"
            if PGPASSWORD="$db_password" $_DEV_COMPOSE_CMD exec -T -e PGPASSWORD db \
                pg_restore -h pgbouncer -p "$pooler_port" -U "$db_user" -d "$db_name" --clean --no-owner --no-acl \
                < "$file" 2>/dev/null; then
                echo -e "${GREEN}✅ Restore complete!${NC}"
            else
                # pg_restore returns non-zero for warnings (e.g. "relation does not exist" on --clean)
                echo -e "${YELLOW}⚠️  Restore finished with warnings (usually harmless with --clean).${NC}"
            fi
            ;;
        sql)
            echo -e "${BLUE}  Restoring (psql)...${NC}"
            if PGPASSWORD="$db_password" $_DEV_COMPOSE_CMD exec -T -e PGPASSWORD db \
                psql -h pgbouncer -p "$pooler_port" -U "$db_user" -d "$db_name" --quiet \
                < "$file" 2>/dev/null; then
                echo -e "${GREEN}✅ Restore complete!${NC}"
            else
                echo -e "${RED}❌ Restore failed!${NC}"
                exit 1
            fi
            ;;
        *)
            echo -e "${RED}❌ Unsupported file format: .$ext (use .dump or .sql)${NC}"
            exit 1
            ;;
    esac
}

