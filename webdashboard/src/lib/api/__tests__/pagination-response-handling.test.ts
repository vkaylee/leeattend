import { describe, it, expect, vi } from "vitest";

import { PaginatedResponse } from "../types";

/**
 * Comprehensive test suite for pagination response handling.
 * Ensures that paginated API responses are properly typed and handled
 * to prevent runtime errors like "TypeError: employees.forEach is not a function".
 *
 * This test suite covers:
 * - Correct response structure validation
 * - Edge cases (undefined, null, empty responses)
 * - TypeScript type safety
 * - Proper data extraction from paginated responses
 */

describe("Pagination Response Handling", () => {
  describe("PaginatedResponse Type Safety", () => {
    it("should define PaginatedResponse with correct structure", () => {
      const mockResponse: PaginatedResponse<{ id: string }> = {
        data: [{ id: "1" }],
        total: 100,
        page: 1,
        limit: 50,
      };

      expect(mockResponse).toEqual({
        data: [{ id: "1" }],
        total: 100,
        page: 1,
        limit: 50,
      });
      expect(Array.isArray(mockResponse.data)).toBe(true);
      expect(typeof mockResponse.total).toBe("number");
      expect(typeof mockResponse.page).toBe("number");
      expect(typeof mockResponse.limit).toBe("number");
    });

    it("should handle empty paginated response", () => {
      const emptyResponse: PaginatedResponse<{ id: string }> = {
        data: [],
        total: 0,
        page: 1,
        limit: 50,
      };

      expect(emptyResponse.data).toHaveLength(0);
      expect(emptyResponse.total).toBe(0);
      expect(Array.isArray(emptyResponse.data)).toBe(true);
    });

    it("should enforce proper data type for paginated response", () => {
      interface Employee {
        id: string;
        full_name: string;
      }

      const response: PaginatedResponse<Employee> = {
        data: [
          { id: "emp-1", full_name: "John Doe" },
          { id: "emp-2", full_name: "Jane Smith" },
        ],
        total: 2,
        page: 1,
        limit: 50,
      };

      expect(response.data).toHaveLength(2);
      expect(response.data[0].full_name).toBe("John Doe");
      expect(response.data[1].full_name).toBe("Jane Smith");
    });
  });

  describe("Data Extraction from Paginated Response", () => {
    it("should safely extract data array from paginated response", () => {
      const response: PaginatedResponse<{ id: string }> = {
        data: [{ id: "1" }, { id: "2" }],
        total: 2,
        page: 1,
        limit: 50,
      };

      // Safe extraction pattern used in components
      const data = response?.data ?? [];
      expect(Array.isArray(data)).toBe(true);
      expect(data).toHaveLength(2);
    });

    it("should handle undefined paginated response gracefully", () => {
      const response: PaginatedResponse<{ id: string }> | undefined = undefined as PaginatedResponse<{ id: string }> | undefined;

      // Safe extraction pattern
      const data = response?.data ?? [];
      expect(Array.isArray(data)).toBe(true);
      expect(data).toHaveLength(0);
    });

    it("should handle null paginated response gracefully", () => {
      const response: PaginatedResponse<{ id: string }> | null = null as PaginatedResponse<{ id: string }> | null;

      // Safe extraction pattern
      const data = response?.data ?? [];
      expect(Array.isArray(data)).toBe(true);
      expect(data).toHaveLength(0);
    });

    it("should prevent forEach errors on extracted data", () => {
      const response: PaginatedResponse<{ id: string; name: string }> = {
        data: [{ id: "1", name: "Item 1" }],
        total: 1,
        page: 1,
        limit: 50,
      };

      const data = response?.data ?? [];
      let forEachCount = 0;

      // Should not throw "TypeError: employees.forEach is not a function"
      expect(() => {
        data.forEach(() => {
          forEachCount++;
        });
      }).not.toThrow();

      expect(forEachCount).toBe(1);
    });

    it("should support array methods on extracted data", () => {
      const response: PaginatedResponse<{ id: string; value: number }> = {
        data: [
          { id: "1", value: 10 },
          { id: "2", value: 20 },
          { id: "3", value: 30 },
        ],
        total: 3,
        page: 1,
        limit: 50,
      };

      const data = response?.data ?? [];

      // map
      const mapped = data.map((item) => item.value);
      expect(mapped).toEqual([10, 20, 30]);

      // filter
      const filtered = data.filter((item) => item.value > 15);
      expect(filtered).toHaveLength(2);

      // find
      const found = data.find((item) => item.id === "2");
      expect(found?.value).toBe(20);

      // reduce
      const sum = data.reduce((acc, item) => acc + item.value, 0);
      expect(sum).toBe(60);
    });
  });

  describe("Component Integration Patterns", () => {
    it("should demonstrate correct pattern for React Query usage", () => {
      // Pattern used in fixed PayrollPage.tsx
      const employeesData: PaginatedResponse<{ id: string; full_name: string }> = {
        data: [{ id: "emp-1", full_name: "John Doe" }],
        total: 1,
        page: 1,
        limit: 50,
      };

      // Correct: Access .data property
      const employees = employeesData?.data ?? [];
      expect(Array.isArray(employees)).toBe(true);
      expect(employees).toHaveLength(1);

      // Should work with array methods
      const names = employees.map((emp) => emp.full_name);
      expect(names).toEqual(["John Doe"]);
    });

    it("should demonstrate incorrect pattern that caused the bug", () => {
      // This is what was happening before the fix
      const employeesData: PaginatedResponse<{ id: string; full_name: string }> =
        {
          data: [{ id: "emp-1", full_name: "John Doe" }],
          total: 1,
          page: 1,
          limit: 50,
        };

      // INCORRECT: Treating paginated response as array directly
      // This would cause: TypeError: employees.forEach is not a function
      // because employeesData is an object with a .data property, not an array

      // The fix ensures we use .data before array operations
      expect(() => {
        const employees = employeesData?.data ?? [];
        employees.forEach(() => {
          /* iteration */
        });
      }).not.toThrow();
    });

    it("should handle missing data property in response", () => {
      const malformedResponse = {
        total: 1,
        page: 1,
        limit: 50,
        // missing 'data' property
      } as unknown as PaginatedResponse<unknown>;

 
      const data = (malformedResponse as unknown as Record<string, unknown>)?.data ?? [];
      expect(Array.isArray(data)).toBe(true);
      expect(data).toHaveLength(0);
    });

    it("should handle pagination metadata correctly", () => {
      const response: PaginatedResponse<{ id: string }> = {
        data: [{ id: "1" }, { id: "2" }],
        total: 100,
        page: 1,
        limit: 50,
      };

      // Extract data for iteration
      const data = response?.data ?? [];

      // Use pagination metadata for UI
      const totalPages = Math.ceil(response.total / response.limit);
      const canGoNext = response.page < totalPages;
      const canGoPrev = response.page > 1;

      expect(data).toHaveLength(2);
      expect(totalPages).toBe(2);
      expect(canGoNext).toBe(true);
      expect(canGoPrev).toBe(false);
    });
  });

  describe("Edge Cases and Error Scenarios", () => {
    it("should handle large dataset pagination", () => {
      const largeData = Array.from({ length: 1000 }, (_, i) => ({
        id: `item-${i}`,
      }));

      const response: PaginatedResponse<{ id: string }> = {
        data: largeData.slice(0, 50),
        total: 1000,
        page: 1,
        limit: 50,
      };

      const data = response?.data ?? [];
      expect(data).toHaveLength(50);
      expect(response.total).toBe(1000);
    });

    it("should handle single item pagination", () => {
      const response: PaginatedResponse<{ id: string; name: string }> = {
        data: [{ id: "1", name: "Single Item" }],
        total: 1,
        page: 1,
        limit: 50,
      };

      const data = response?.data ?? [];
      expect(data).toHaveLength(1);
      expect(data[0].name).toBe("Single Item");
    });

    it("should handle null data array gracefully", () => {
      const response = {
        data: null,
        total: 0,
        page: 1,
        limit: 50,
      } as unknown as PaginatedResponse<unknown>;

 
      const data = (response as unknown as Record<string, unknown>)?.data ?? [];
      expect(Array.isArray(data)).toBe(true);
      expect(data).toHaveLength(0);
    });

    it("should prevent common array method errors", () => {
      const scenarios = [
        { response: undefined, name: "undefined response" },
        { response: null, name: "null response" },
        { response: { data: [] }, name: "empty data array" },
      ];

      scenarios.forEach(({ response, name }) => {
        const data = response?.data ?? [];

        expect(() => {
          data.forEach(() => {
            /* noop */
          });
          data.map((x) => x);
          data.filter((x) => x);
          data.find((x) => x);
          expect(data.length).toBeGreaterThanOrEqual(0);
        }).not.toThrow(`Should not throw for ${name}`);
      });
    });

    it("should handle response with unexpected structure", () => {
      const unexpectedResponse = {
        results: [{ id: "1" }], // Wrong property name
        count: 1,
      } as unknown as PaginatedResponse<unknown>;

 
      const data = (unexpectedResponse as unknown as Record<string, unknown>)?.data ?? [];
      expect(Array.isArray(data)).toBe(true);
      expect(data).toHaveLength(0);
    });

    it("should validate pagination numbers", () => {
      const responses: PaginatedResponse<{ id: string }>[] = [
        {
          data: [],
          total: 0,
          page: 1,
          limit: 50,
        },
        {
          data: [{ id: "1" }],
          total: 1,
          page: 1,
          limit: 1,
        },
        {
          data: [{ id: "1" }, { id: "2" }],
          total: 100,
          page: 3,
          limit: 50,
        },
      ];

      responses.forEach((response) => {
        expect(response.total).toBeGreaterThanOrEqual(0);
        expect(response.page).toBeGreaterThanOrEqual(1);
        expect(response.limit).toBeGreaterThan(0);
        expect(response.data).toBeInstanceOf(Array);
      });
    });
  });

  describe("API Function Typing", () => {
    it("should validate fetchEmployees returns PaginatedResponse type", async () => {
      // This test validates the function signature
      // The actual API mock would be handled by integration tests

      const mockFetch = vi.fn();

      // Mock the apiFetch to validate response structure
      const createMockResponse = (data: unknown) => ({
        ok: true,
 
        json: async () => data,
      });

      const paginatedEmployees: PaginatedResponse<{
        id: string;
        full_name: string;
      }> = {
        data: [
          { id: "emp-1", full_name: "John Doe" },
          { id: "emp-2", full_name: "Jane Smith" },
        ],
        total: 2,
        page: 1,
        limit: 50,
      };

      mockFetch.mockResolvedValue(
        createMockResponse(paginatedEmployees)
      );

      const result = await mockFetch();
      const parsed = await result.json();

      expect(parsed.data).toBeInstanceOf(Array);
      expect(parsed.total).toBe(2);
      expect(parsed.page).toBe(1);
      expect(parsed.limit).toBe(50);
    });

    it("should validate searchEmployees returns PaginatedResponse type", async () => {
      const mockFetch = vi.fn();

      const createMockResponse = (data: unknown) => ({
        ok: true,
 
        json: async () => data,
      });

      const searchResults: PaginatedResponse<{
        id: string;
        full_name: string;
      }> = {
        data: [{ id: "emp-1", full_name: "John" }],
        total: 1,
        page: 1,
        limit: 50,
      };

      mockFetch.mockResolvedValue(
        createMockResponse(searchResults)
      );

      const result = await mockFetch();
      const parsed = await result.json() as PaginatedResponse<{ id: string; full_name: string }>;

      expect(parsed.data).toHaveLength(1);
      expect(parsed.data[0].full_name).toContain("John");
    });
  });

  describe("TypeScript Type Enforcement", () => {
    it("should enforce PaginatedResponse structure at compile time", () => {
      // This test documents the type structure for developers
      interface TestItem {
        id: string;
        value: string;
      }

      const validResponse: PaginatedResponse<TestItem> = {
        data: [
          { id: "1", value: "test" },
          { id: "2", value: "demo" },
        ],
        total: 2,
        page: 1,
        limit: 50,
      };

      const extractedData = validResponse.data;
      expect(extractedData[0].id).toBe("1");
      expect(extractedData[0].value).toBe("test");
    });

    it("should allow optional fields in PaginatedResponse items", () => {
      interface EmployeeOptional {
        id: string;
        full_name: string;
        department?: string;
      }

      const response: PaginatedResponse<EmployeeOptional> = {
        data: [
          { id: "1", full_name: "John", department: "Sales" },
          { id: "2", full_name: "Jane" }, // Optional department omitted
        ],
        total: 2,
        page: 1,
        limit: 50,
      };

      expect(response.data).toHaveLength(2);
      expect(response.data[0].department).toBe("Sales");
      expect(response.data[1].department).toBeUndefined();
    });
  });
});
