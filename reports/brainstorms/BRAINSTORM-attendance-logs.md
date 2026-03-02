## Requirements Discovery: Cải thiện UI/UX tìm kiếm & điều hướng tại /attendance-logs

### Initial Request
Ở trang /attendance-logs, hãy cải thiện UIUX vì mục đích người dùng vào trang này là để tìm attendance logs của một nhân viên nào đó, admin cần focus vào ô filter tìm kiếm nhân viên, khi họ thấy người họ cần tìm đang ở trên đầu và nhấn enter là chọn người đó, sau đó họ muốn xem người khác thì phải có cách để họ có thể dùng phím và gỏ tiếp

### Clarifying Questions
1. Q: "Focus vào ô filter tìm kiếm nhân viên" nghĩa là focus tự động khi trang vừa load hay khi người dùng bấm phím tắt nào đó?
   A: Tạm định nghĩa là tự động focus khi trang load để tối ưu thao tác (cần xác nhận lại nếu hệ thống có phím tắt chuẩn).
2. Q: "Nhấn enter là chọn người đó đang ở trên đầu" - người trên đầu là bản ghi đầu tiên trong bảng kết quả (table) hay là mục đầu tiên trong dropdown suggest của ô tìm kiếm?
   A: Dựa trên mô tả, có vẻ đây là ô Select/Dropdown hoặc một dạng Data Table có filter. Cần xem mã nguồn để biết chính xác là component nào.
3. Q: Sau khi chọn xong người đó, làm sao để xem người khác bằng bàn phím mà không cần chuột?
   A: Cần duy trì focus ở ô tìm kiếm (hoặc cho phép phím ESC/phím tắt chọn lại ô tìm kiếm), để gõ tiếp từ khóa mới và lặp lại thao tác.

### Problem Statement
Trang `/attendance-logs` hiện tại chưa tối ưu cho thao tác sử dụng bàn phím (keyboard navigation). Quản trị viên phải dùng chuột nhiều để tìm, chọn và chuyển đổi giữa các nhân viên khác nhau khi xem lịch sử điểm danh. Cần làm cho luồng thao tác (nhập → kết quả tự động filter → Enter chọn người đầu tiên → xem thông tin → gõ tiếp để tìm người khác) hoàn toàn dùng được bằng bàn phím (keyboard-first experience).

### Stakeholders
| Role | Needs | Priority |
| ---- | ----- | -------- |
| Admin / HR | Tra cứu điểm danh nhanh, liên tục cho nhiều nhân sự hiệu quả bằng bàn phím, hạn chế dùng chuột | H |

### Requirements
#### Functional
| ID | Requirement | Priority |
| --- | ----------- | -------- |
| FR1 | Auto-focus vào ô Input/Filter tìm kiếm nhân viên khi trang /attendance-logs được tải. | Must |
| FR2 | Khi gõ tìm kiếm và có danh sách kết quả (ở list hoặc table), nhấn Enter sẽ tự động kích hoạt thao tác "Chọn" đối với kết quả đầu tiên trên cùng. | Must |
| FR3 | Sau khi đã "Chọn" và đang xem dữ liệu của 1 người, việc gõ phím text (alphanumeric) hoặc nhấn phím tắt (VD: Esc hoặc /) sẽ lập tức đưa focus quay lại ô Input tìm kiếm. Ô Input này không bị mất đi trạng thái thao tác. | Must |
| FR4 | Giá trị cũ trong ô Input khi focus lại nên được bôi đen (select all) hoặc xóa trống nhanh để có thể gõ ngay tên người mới. | Should |

### Success Criteria
1. Mở trang `/attendance-logs`, không cần click chuột vẫn có thể gõ ngay tên nhân viên.
2. Gõ "Nguyễn Văn A", nhấn Enter -> Xem được log của nhân viên A (nếu A đứng đầu danh sách).
3. Ngay sau đó, gõ tiếp "Trần Văn B" -> Hệ thống tự động focus lại ô tìm kiếm, lọc và nhấn Enter -> Xem được log của nhân viên B.

### Open Questions
- Component "ô filter" hiện tại đang được code như thế nào? (Là input text thường hay Shadcn/Radix select, combobox?)
- Khi "chọn" nhân viên, giao diện xử lý thế nào? (Chuyển trang, hiện modal, hay load lại bảng ở dưới?) Cần khảo sát mã nguồn để implement chính xác behaviour.
