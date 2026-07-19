export interface UploadResult {
  errors: {
    line_number: number;
    employment_id: string;
    error_message: string;
  }[];
  failed_count: number;
  success_count: number;
  total_rows: number;
}
