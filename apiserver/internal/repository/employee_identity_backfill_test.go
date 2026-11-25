package repository

import (
	"context"
	"errors"
	"testing"

	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
)

func TestBackfillIdentityLookupHashesFailsClosedWithoutSecret(t *testing.T) {
	repo := EmployeeIdentityRepository{}
	var zeroHasher identity.Hasher
	_, err := repo.BackfillIdentityLookupHashes(context.Background(), zeroHasher, IdentityBackfillOptions{})
	if !errors.Is(err, identity.ErrIdentityHashSecretMissing) {
		t.Fatalf("err = %v, want ErrIdentityHashSecretMissing", err)
	}
}

func TestBackfillIdentityLookupHashesValidatesBatchSize(t *testing.T) {
	hasher, err := identity.NewHasher("0123456789abcdef0123456789abcdef")
	if err != nil {
		t.Fatalf("NewHasher: %v", err)
	}
	repo := EmployeeIdentityRepository{}
	for _, bad := range []int{-1, 1001} {
		_, err := repo.BackfillIdentityLookupHashes(context.Background(), hasher, IdentityBackfillOptions{BatchSize: bad})
		if !errors.Is(err, ErrIdentityBackfillInvalidBatchSize) {
			t.Fatalf("BatchSize=%d err = %v, want ErrIdentityBackfillInvalidBatchSize", bad, err)
		}
	}
}

func TestCountCollisionsAggregateOnly(t *testing.T) {
	groups, rows := countCollisions(map[string]int{"a": 1, "b": 2, "c": 3})
	if groups != 2 || rows != 5 {
		t.Fatalf("countCollisions = (%d, %d), want (2, 5)", groups, rows)
	}
}
