## Brainstorm: UX Improvements cho /attendance-logs

### Current State
- Trang có 2 thẻ (Tabs): "Recorded Logs" và "Unidentified".
- Filter có các mốc thời gian: "this_month", "last_month", "specific_month", "custom".
- Tìm kiếm nhân sự qua `EmployeeSearch` (vừa được cải thiện keyboard navigation với `Enter` và `/`).
- Có nút Export, Import, View Reports.
- Bảng kết quả (Table) hiển thị Time, Employee, Source, Status, và phân trang.

### Proposed UX Improvements

1. **Hiển thị gợi ý phím tắt (Keyboard Hints)**:
   - Thêm tooltip hoặc text nhỏ mờ mờ ở form tìm kiếm: `Press / or Enter to search` để người dùng biết tính năng tìm kiếm nhanh bằng bàn phím vừa được làm.

2. **Loading Skeleton thay vì chữ mộc**:
   - Hiện tại đang dùng `<tr><td colSpan={4}>Loading...</td></tr>`. Có thể thay bằng Skeleton loading row (hiệu ứng shimmer) để mượt mắt hơn trong lúc chờ data.

3. **Empty State chuẩn hơn**:
   - Hiện tại lúc không có dữ liệu chỉ có dòng chữ: `No data`. Thay bằng một Empty State block có icon minh hoạ lớn, text thân thiện hơn, hoặc gợi ý chọn lại filter.

4. **Nổi bật bộ lọc hiện tại / Nút Clear Filter**:
   - Nut "All Employees" hiện tại (cái nút hình `Users` kế bên `EmployeeSearch`) có thể thêm UI rõ ràng hơn dạng "Clear filter (Esc)" hoặc hiển thị badge "Filtering by: Nguyen Van A" và nút X để tắt filter.

5. **Sticky Table Header / Sticky Filter**:
   - Nếu dữ liệu dài (nhiều rows), tiêu đề bảng `<thead>` hoặc thanh công cụ filter (chứa tìm người & các nút filter) nên làm dạng `sticky top-0 z-10` để người dùng cuộn xuống dưới vẫn nhìn thấy bộ lọc và các tiêu đề cột.

6. **Highlight (Nhấn mạnh) trạng thái**:
   - Trong tab "Unidentified", khi thẻ "Assign" hay "Create" được bấm, UI đang chuyển sang ô select. Có thể thêm hiệu ứng focus-within mịn màng hơn khi gán nhân viên.

### Priority (Sắp xếp theo độ khả thi nhanh)
1. Thêm Keyboard Hint vào UI (`Press / to search`).
2. Đổi chữ "Loading..." thành Skeleton UI cho UX premium.
3. Làm đẹp trạng thái "No data" (Empty state) cho cả 2 tab.
4. Gắn `sticky` cho thead hoặc block filter.
