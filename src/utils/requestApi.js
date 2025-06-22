// 基础配置
const BASE_URL = ''; // 使用相对路径，通过Vite代理转发

// 请求头配置
const getHeaders = (contentType = 'application/json') => ({
    'Content-Type': contentType,
    // 如果需要认证，可以在这里添加 token
    // 'Authorization': `Bearer ${getToken()}`,
});

// 处理响应
const handleResponse = async (response) => {
    if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || `HTTP error! status: ${response.status}`);
    }
    return response.json();
};

// 处理错误
const handleError = (error) => {
    console.error('API请求错误:', error);
    throw error;
};

// GET 请求
export const get = async (endpoint, params = {}) => {
    try {
        // 构建查询参数
        const queryString = new URLSearchParams(params).toString();
        const url = `${BASE_URL}${endpoint}${queryString ? `?${queryString}` : ''}`;
        
        const response = await fetch(url, {
            method: 'GET',
            headers: getHeaders(),
        });
        
        return await handleResponse(response);
    } catch (error) {
        handleError(error);
    }
};

// POST 请求
export const post = async (endpoint, data = {}) => {
    try {
        const response = await fetch(`${BASE_URL}${endpoint}`, {
            method: 'POST',
            headers: getHeaders(),
            body: JSON.stringify(data),
        });
        
        return await handleResponse(response);
    } catch (error) {
        handleError(error);
    }
};

// PUT 请求
export const put = async (endpoint, data = {}) => {
    try {
        const response = await fetch(`${BASE_URL}${endpoint}`, {
            method: 'PUT',
            headers: getHeaders(),
            body: JSON.stringify(data),
        });
        
        return await handleResponse(response);
    } catch (error) {
        handleError(error);
    }
};

// DELETE 请求
export const del = async (endpoint) => {
    try {
        const response = await fetch(`${BASE_URL}${endpoint}`, {
            method: 'DELETE',
            headers: getHeaders(),
        });
        
        return await handleResponse(response);
    } catch (error) {
        handleError(error);
    }
};

// 通用请求方法
export const request = async (endpoint, options = {}) => {
    try {
        const {
            method = 'GET',
            data = null,
            params = {},
            headers = {},
        } = options;

        // 构建 URL
        const queryString = new URLSearchParams(params).toString();
        const url = `${BASE_URL}${endpoint}${queryString ? `?${queryString}` : ''}`;

        // 构建请求配置
        const config = {
            method: method.toUpperCase(),
            headers: {
                ...getHeaders(),
                ...headers,
            },
        };

        // 添加请求体
        if (data && ['POST', 'PUT', 'PATCH'].includes(method.toUpperCase())) {
            config.body = JSON.stringify(data);
        }

        const response = await fetch(url, config);
        return await handleResponse(response);
    } catch (error) {
        handleError(error);
    }
};

// 保留原有的 fetchApi 函数以保持兼容性
export const fetchApi = async (url, options) => {
    const response = await fetch(url, options);
    const data = await response.json();
    return data;
};

