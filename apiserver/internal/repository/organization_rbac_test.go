package repository

import (
	"context"
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type rbacTestRows struct {
	rows [][]any
	idx  int
}

func (r *rbacTestRows) Close()                                       {}
func (r *rbacTestRows) Err() error                                   { return nil }
func (r *rbacTestRows) CommandTag() pgconn.CommandTag                { return pgconn.CommandTag{} }
func (r *rbacTestRows) FieldDescriptions() []pgconn.FieldDescription { return nil }
func (r *rbacTestRows) Next() bool {
	if r.idx < len(r.rows) {
		r.idx++
		return true
	}
	return false
}
func (r *rbacTestRows) Scan(dest ...any) error {
	row := r.rows[r.idx-1]
	for i, v := range row {
		switch d := dest[i].(type) {
		case *uuid.UUID:
			*d = v.(uuid.UUID)
		case **uuid.UUID:
			if v == nil {
				*d = nil
			} else {
				u := v.(uuid.UUID)
				*d = &u
			}
		case *string:
			*d = v.(string)
		case **string:
			if v == nil {
				*d = nil
			} else if sp, ok := v.(*string); ok {
				*d = sp
			} else {
				s := v.(string)
				*d = &s
			}
		case *bool:
			*d = v.(bool)
		case *int:
			*d = v.(int)
		case *int64:
			*d = v.(int64)
		case *time.Time:
			*d = v.(time.Time)
		case **time.Time:
			if v == nil {
				*d = nil
			} else {
				t := v.(time.Time)
				*d = &t
			}
		case *api.RoleScopeType:
			*d = v.(api.RoleScopeType)
		}
	}
	return nil
}
func (r *rbacTestRows) Values() ([]any, error) { return nil, nil }
func (r *rbacTestRows) RawValues() [][]byte    { return nil }
func (r *rbacTestRows) Conn() *pgx.Conn        { return nil }

func TestOrganizationRBACRepository_RequiresDatabase(t *testing.T) {
	repo := OrganizationRBACRepository{}
	ctx := context.Background()
	uID, rID := uuid.New(), uuid.New()

	if _, _, err := repo.ListMembers(ctx, 1, 10); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.ListExtraRoles(ctx); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.FindUserByEmail(ctx, "a@b.com"); err == nil {
		t.Fatal("expected nil db error")
	}
	if err := repo.AddMember(ctx, uID, rID); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.FindPrimaryRole(ctx, uID); err == nil {
		t.Fatal("expected nil db error")
	}
	if err := repo.UpdateMemberRole(ctx, uID, rID); err == nil {
		t.Fatal("expected nil db error")
	}
	if err := repo.RemoveMember(ctx, uID); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, _, err := repo.ListMemberRoles(ctx, uID); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.AssignExtraRole(ctx, uID, rID, time.Now()); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.RevokeExtraRole(ctx, rID); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.ListMemberPermissions(ctx, uID); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.AssignExtraPermission(ctx, uID, "perm", time.Now(), uID); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.RevokeExtraPermission(ctx, rID); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, _, err := repo.ListRoles(ctx, 1, 10); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.CreateRole(ctx, api.CreateRoleRequest{}); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.UpdateRole(ctx, rID, api.UpdateRoleRequest{}); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.DeleteRole(ctx, rID); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.RolePermissions(ctx, rID); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.RoleIsMutable(ctx, rID); err == nil {
		t.Fatal("expected nil db error")
	}
	if err := repo.SetRolePermissions(ctx, rID, []string{"perm"}); err == nil {
		t.Fatal("expected nil db error")
	}
}

func TestOrganizationRBACRepository_Members_ListAndAdd(t *testing.T) {
	ctx := context.Background()
	userID := uuid.New()
	roleID := uuid.New()
	email := "alice@example.com"
	fullName := "Alice"

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			if strings.Contains(sql, "SELECT COUNT(*) FROM user_roles") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*int64)) = 1
					return nil
				}}
			}
			if strings.Contains(sql, "FROM users WHERE LOWER(email)") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = userID
					return nil
				}}
			}
			if strings.Contains(sql, "SELECT role_id FROM user_roles") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = roleID
					return nil
				}}
			}
			return fakeRow{}
		},
		queryFn: func(_ context.Context, sql string, args ...any) (pgx.Rows, error) {
			if strings.Contains(sql, "FROM user_roles ur") && strings.Contains(sql, "JOIN users u") {
				return &rbacTestRows{
					rows: [][]any{
						{userID, roleID, email, &fullName, (*string)(nil)},
					},
				}, nil
			}
			return &rbacTestRows{}, nil
		},
		execFn: func(_ context.Context, sql string, arguments ...any) (pgconn.CommandTag, error) {
			if strings.Contains(sql, "INSERT INTO user_roles") {
				return pgconn.NewCommandTag("INSERT 0 1"), nil
			}
			return pgconn.CommandTag{}, nil
		},
	}

	repo := OrganizationRBACRepository{SystemDB: db}

	members, total, err := repo.ListMembers(ctx, 1, 10)
	if err != nil || total != 1 || len(members) != 1 {
		t.Fatalf("ListMembers failed: err=%v, total=%d, count=%d", err, total, len(members))
	}
	if members[0].UserID != userID || members[0].RoleID != roleID || members[0].Email != email {
		t.Fatalf("unexpected member detail: %+v", members[0])
	}

	foundID, err := repo.FindUserByEmail(ctx, email)
	if err != nil || foundID != userID {
		t.Fatalf("FindUserByEmail failed: %v", err)
	}

	primary, err := repo.FindPrimaryRole(ctx, userID)
	if err != nil || primary == nil || *primary != roleID {
		t.Fatalf("FindPrimaryRole failed: %v", err)
	}

	if err := repo.AddMember(ctx, userID, roleID); err != nil {
		t.Fatalf("AddMember failed: %v", err)
	}
}

func TestOrganizationRBACRepository_UpdateMemberRole_LastAdminGuard(t *testing.T) {
	ctx := context.Background()
	adminUser := uuid.New()
	adminRoleID := uuid.New()
	scopedRoleID := uuid.New()

	var adminCount int64 = 1

	tx := &fakeTx{
		fakeDBTX: fakeDBTX{
			queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
				if strings.Contains(sql, "SELECT scope_type, name FROM roles WHERE id = $1") {
					targetRole := args[0].(uuid.UUID)
					if targetRole == scopedRoleID {
						return fakeRow{scanFn: func(dest ...any) error {
							*(dest[0].(*string)) = "scoped"
							*(dest[1].(*string)) = "employee"
							return nil
						}}
					}
					if targetRole == adminRoleID {
						return fakeRow{scanFn: func(dest ...any) error {
							*(dest[0].(*string)) = "global"
							*(dest[1].(*string)) = "admin"
							return nil
						}}
					}
					return fakeRow{scanFn: func(...any) error { return pgx.ErrNoRows }}
				}
				if strings.Contains(sql, "SELECT ur.role_id, r.scope_type, r.name FROM user_roles ur") {
					return fakeRow{scanFn: func(dest ...any) error {
						*(dest[0].(*uuid.UUID)) = adminRoleID
						*(dest[1].(*string)) = "global"
						*(dest[2].(*string)) = "admin"
						return nil
					}}
				}
				if strings.Contains(sql, "SELECT EXISTS (SELECT 1 FROM user_extra_roles") {
					return fakeRow{scanFn: func(dest ...any) error {
						*(dest[0].(*bool)) = false
						return nil
					}}
				}
				if strings.Contains(sql, "SELECT COUNT(DISTINCT user_id) FROM") {
					return fakeRow{scanFn: func(dest ...any) error {
						*(dest[0].(*int64)) = adminCount
						return nil
					}}
				}
				return fakeRow{}
			},
			execFn: func(_ context.Context, sql string, arguments ...any) (pgconn.CommandTag, error) {
				if strings.Contains(sql, "UPDATE user_roles SET role_id = $1 WHERE user_id = $2") {
					return pgconn.NewCommandTag("UPDATE 1"), nil
				}
				return pgconn.CommandTag{}, nil
			},
		},
	}

	repo := OrganizationRBACRepository{
		SystemDB: tx,
		BeginTx: func(context.Context) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	// 1. Demoting last global admin must be rejected
	adminCount = 1
	err := repo.UpdateMemberRole(ctx, adminUser, scopedRoleID)
	if !errors.Is(err, api.ErrRBACLastAdmin) {
		t.Fatalf("expected ErrRBACLastAdmin when demoting last admin, got %v", err)
	}

	// 2. Demoting when multiple global admins exist must succeed
	adminCount = 2
	err = repo.UpdateMemberRole(ctx, adminUser, scopedRoleID)
	if err != nil {
		t.Fatalf("unexpected error when multiple admins exist: %v", err)
	}
}

func TestOrganizationRBACRepository_RemoveMember_LastAdminGuard(t *testing.T) {
	ctx := context.Background()
	adminUser := uuid.New()
	var adminCount int64 = 1

	tx := &fakeTx{
		fakeDBTX: fakeDBTX{
			queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
				if strings.Contains(sql, "SELECT r.scope_type, r.name FROM user_roles ur") {
					return fakeRow{scanFn: func(dest ...any) error {
						*(dest[0].(*string)) = "global"
						*(dest[1].(*string)) = "admin"
						return nil
					}}
				}
				if strings.Contains(sql, "SELECT EXISTS (SELECT 1 FROM user_extra_roles") {
					return fakeRow{scanFn: func(dest ...any) error {
						*(dest[0].(*bool)) = false
						return nil
					}}
				}
				if strings.Contains(sql, "SELECT COUNT(DISTINCT user_id) FROM") {
					return fakeRow{scanFn: func(dest ...any) error {
						*(dest[0].(*int64)) = adminCount
						return nil
					}}
				}
				return fakeRow{}
			},
			execFn: func(_ context.Context, sql string, arguments ...any) (pgconn.CommandTag, error) {
				if strings.Contains(sql, "DELETE FROM user_roles WHERE user_id = $1") {
					return pgconn.NewCommandTag("DELETE 1"), nil
				}
				return pgconn.NewCommandTag("DELETE 0"), nil
			},
		},
	}

	repo := OrganizationRBACRepository{
		SystemDB: tx,
		BeginTx: func(context.Context) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	// 1. Removing last global admin must be rejected
	adminCount = 1
	err := repo.RemoveMember(ctx, adminUser)
	if !errors.Is(err, api.ErrRBACLastAdmin) {
		t.Fatalf("expected ErrRBACLastAdmin when removing last admin, got %v", err)
	}

	// 2. Removing when multiple global admins exist must succeed
	adminCount = 2
	err = repo.RemoveMember(ctx, adminUser)
	if err != nil {
		t.Fatalf("unexpected error when removing admin with multiple existing: %v", err)
	}
}

func TestOrganizationRBACRepository_RolesAndPermissions(t *testing.T) {
	ctx := context.Background()
	systemRoleID := uuid.New()
	customRoleID := uuid.New()
	now := time.Now().UTC()

	tx := &fakeTx{
		fakeDBTX: fakeDBTX{
			queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
				if strings.Contains(sql, "SELECT COUNT(*) FROM roles") {
					return fakeRow{scanFn: func(dest ...any) error {
						*(dest[0].(*int64)) = 2
						return nil
					}}
				}
				if strings.Contains(sql, "SELECT is_system FROM roles WHERE id = $1") {
					roleID := args[0].(uuid.UUID)
					if roleID == systemRoleID {
						return fakeRow{scanFn: func(dest ...any) error {
							*(dest[0].(*bool)) = true
							return nil
						}}
					}
					return fakeRow{scanFn: func(dest ...any) error {
						*(dest[0].(*bool)) = false
						return nil
					}}
				}
				if strings.Contains(sql, "SELECT EXISTS (SELECT 1 FROM roles WHERE id = $1 AND is_system = false)") {
					roleID := args[0].(uuid.UUID)
					return fakeRow{scanFn: func(dest ...any) error {
						*(dest[0].(*bool)) = roleID == customRoleID
						return nil
					}}
				}
				if strings.Contains(sql, "INSERT INTO roles") {
					return fakeRow{scanFn: func(dest ...any) error {
						*(dest[0].(*uuid.UUID)) = customRoleID
						*(dest[1].(*string)) = "Custom"
						*(dest[2].(**string)) = nil
						*(dest[3].(*string)) = "blue"
						*(dest[4].(*bool)) = false
						*(dest[5].(*api.RoleScopeType)) = api.RoleScopeScoped
						*(dest[6].(*time.Time)) = now
						*(dest[7].(*time.Time)) = now
						return nil
					}}
				}
				return fakeRow{}
			},
			queryFn: func(_ context.Context, sql string, args ...any) (pgx.Rows, error) {
				if strings.Contains(sql, "FROM roles ORDER BY") {
					return &rbacTestRows{
						rows: [][]any{
							{systemRoleID, "admin", (*string)(nil), "rose", true, api.RoleScopeGlobal, now, now},
							{customRoleID, "custom", (*string)(nil), "blue", false, api.RoleScopeScoped, now, now},
						},
					}, nil
				}
				if strings.Contains(sql, "FROM role_permissions") {
					return &rbacTestRows{
						rows: [][]any{
							{"organization:settings:read"},
							{"members:read"},
						},
					}, nil
				}
				return &rbacTestRows{}, nil
			},
			execFn: func(_ context.Context, sql string, arguments ...any) (pgconn.CommandTag, error) {
				if strings.Contains(sql, "DELETE FROM roles WHERE id = $1 AND is_system = false") {
					roleID := arguments[0].(uuid.UUID)
					if roleID == customRoleID {
						return pgconn.NewCommandTag("DELETE 1"), nil
					}
					return pgconn.NewCommandTag("DELETE 0"), nil
				}
				return pgconn.NewCommandTag("EXEC 1"), nil
			},
		},
	}

	repo := OrganizationRBACRepository{
		SystemDB: tx,
		BeginTx: func(context.Context) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	roles, total, err := repo.ListRoles(ctx, 1, 10)
	if err != nil || total != 2 || len(roles) != 2 {
		t.Fatalf("ListRoles failed: %v", err)
	}

	created, err := repo.CreateRole(ctx, api.CreateRoleRequest{Name: "Custom", Color: "blue", ScopeType: api.RoleScopeScoped})
	if err != nil || created.ID != customRoleID {
		t.Fatalf("CreateRole failed: %v", err)
	}

	isMutable, err := repo.RoleIsMutable(ctx, customRoleID)
	if err != nil || !isMutable {
		t.Fatalf("expected custom role to be mutable: %v", err)
	}

	isMutable, err = repo.RoleIsMutable(ctx, systemRoleID)
	if err != nil || isMutable {
		t.Fatalf("expected system role not to be mutable: %v", err)
	}

	perms, err := repo.RolePermissions(ctx, customRoleID)
	if err != nil || len(perms) != 2 {
		t.Fatalf("RolePermissions failed: %v", err)
	}

	// SetRolePermissions on system role must be rejected
	err = repo.SetRolePermissions(ctx, systemRoleID, []string{"members:read"})
	if !errors.Is(err, api.ErrRBACProtected) {
		t.Fatalf("expected ErrRBACProtected on system role SetRolePermissions, got %v", err)
	}

	// SetRolePermissions on custom role must succeed
	err = repo.SetRolePermissions(ctx, customRoleID, []string{"members:read"})
	if err != nil {
		t.Fatalf("SetRolePermissions failed on custom role: %v", err)
	}

	// DeleteRole
	deleted, err := repo.DeleteRole(ctx, customRoleID)
	if err != nil || deleted != 1 {
		t.Fatalf("DeleteRole failed: %v, deleted=%d", err, deleted)
	}
}
