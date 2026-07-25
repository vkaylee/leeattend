export interface FormState {
  accessKey: string;
  bucketName: string;
  endpointUrl: string;
  isCustom: boolean;
  region: string;
  secretKey: string;
  usePathStyle: boolean;
}

export const EMPTY_FORM: FormState = {
  isCustom: false,
  endpointUrl: "",
  region: "",
  bucketName: "",
  accessKey: "",
  secretKey: "",
  usePathStyle: true,
};
