import { API_BASE_URL } from "@/lib/api/config";
import { ApiError, AppErrorResponse } from "@/lib/api/errors";
import { apiFetch } from "@/lib/api/helpers";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const customFetch = async <T>(configOrUrl: any, extraOptions?: RequestInit): Promise<T> => {
  let finalUrl: string;
  let options: RequestInit;

  if (typeof configOrUrl === "string") {
    // Orval fetch client passes (url, options)
    finalUrl = `${API_BASE_URL}${configOrUrl}`;
    options = { ...extraOptions };
  } else {
    // Legacy / object config support
    const { url, method, params, data, headers } = configOrUrl;
    const urlObj = new URL(`${API_BASE_URL}${url}`, typeof window !== "undefined" ? window.location.origin : "http://localhost");
    
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          urlObj.searchParams.append(key, String(value));
        }
      });
    }
    finalUrl = urlObj.toString();

    options = {
      method: method ? method.toUpperCase() : "GET",
      headers: {
        "Content-Type": "application/json",
        ...headers,
        ...extraOptions?.headers,
      },
      ...extraOptions,
    };

    if (data) {
      options.body = JSON.stringify(data);
    }
  }

  const res = await apiFetch(finalUrl, options);

  if (!res.ok) {
    let errorBody: AppErrorResponse | undefined;
    let errorText = "";

    try {
      errorText = await res.text();
      if (errorText) {
        errorBody = JSON.parse(errorText) as AppErrorResponse;
      }
    } catch {
      // Ignore parsing errors, keep raw text or undefined body
    }

    throw new ApiError(
      res.status,
      res.statusText,
      errorBody?.error_code,
      errorBody ?? (errorText ? { message: errorText } : undefined)
    );
  }

  const text = await res.text();
  const parsedData = text ? JSON.parse(text) : null;
  
  // Return an object matching AxiosResponse/Orval's expected format
  return {
    data: parsedData,
    status: res.status,
    headers: res.headers,
  } as unknown as T;
};

export default customFetch;
