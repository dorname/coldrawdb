import { actixGet, actixPost, actixPut, actixDel, request } from './requestApi.js';

// 任务相关的 API 调用示例
export const taskApi = {
    // 获取所有任务
    getAllTasks: async () => {
        return await actixGet('/api/tasks');
    },

    // 根据ID获取任务
    getTaskById: async (id) => {
        return await actixGet(`/api/tasks/${id}`);
    },

    // 根据条件查询任务
    getTasksByCondition: async (params) => {
        return await actixGet('/api/tasks', params);
    },

    // 创建新任务
    createTask: async (taskData) => {
        return await actixPost('/api/tasks', taskData);
    },

    // 更新任务
    updateTask: async (id, taskData) => {
        return await actixPut(`/api/tasks/${id}`, taskData);
    },

    // 删除任务
    deleteTask: async (id) => {
        return await actixDel(`/api/tasks/${id}`);
    },

    // 批量插入任务
    insertTasks: async (tasks) => {
        return await actixPost('/api/insert_tasks', { tasks });
    }
};

// 图表相关的 API 调用示例
export const diagramApi = {
    // 获取所有图表
    getAllDiagrams: async () => {
        return await actixGet('/api/diagrams');
    },

    // 获取最新图表
    getLatestDiagram: async () => {
        return await actixGet('/api/diagrams/latest');
    },

    // 根据ID获取图表
    getDiagramById: async (id) => {
        return await actixGet(`/api/diagrams/${id}`);
    },

    // 创建新图表
    createDiagram: async (diagramData) => {
        return await actixPost('/api/diagrams', diagramData);
    },

    // 更新图表
    updateDiagram: async (id, diagramData) => {
        return await actixPut(`/api/diagrams/${id}`, diagramData);
    },

    // 删除图表
    deleteDiagram: async (id) => {
        return await actixDel(`/api/diagrams/${id}`);
    }
};

// 使用通用 request 方法的示例
export const advancedApi = {
    // 带查询参数的请求
    searchTasks: async (keyword, priority, page = 1, limit = 10) => {
        return await request('/api/tasks/search', {
            method: 'GET',
            params: {
                keyword,
                priority,
                page,
                limit
            }
        });
    },

    // 带自定义请求头的请求
    uploadFile: async (file, customHeaders = {}) => {
        const formData = new FormData();
        formData.append('file', file);
        
        return await request('/api/upload', {
            method: 'POST',
            data: formData,
            headers: {
                ...customHeaders,
                // 注意：上传文件时不要设置 Content-Type，让浏览器自动设置
                'Content-Type': undefined
            }
        });
    },

    // 带认证的请求
    getProtectedData: async (token) => {
        return await request('/api/protected/data', {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
    }
};

// 使用示例
export const usageExamples = {
    // 基本使用
    basicUsage: async () => {
        try {
            // 获取所有任务
            const tasks = await taskApi.getAllTasks();
            console.log('所有任务:', tasks);

            // 创建新任务
            const newTask = await taskApi.createTask({
                title: '新任务',
                details: '任务详情',
                priority: 2,
                task_order: 1,
                compele: false
            });
            console.log('创建的任务:', newTask);

            // 更新任务
            const updatedTask = await taskApi.updateTask(newTask.id, {
                compele: true
            });
            console.log('更新的任务:', updatedTask);

        } catch (error) {
            console.error('操作失败:', error.message);
        }
    },

    // 高级使用
    advancedUsage: async () => {
        try {
            // 搜索任务
            const searchResults = await advancedApi.searchTasks(
                '重要',  // 关键词
                3,       // 高优先级
                1,       // 第一页
                20       // 每页20条
            );
            console.log('搜索结果:', searchResults);

            // 批量插入任务
            const tasksToInsert = [
                { title: '任务1', details: '详情1', priority: 1, task_order: 1, compele: false },
                { title: '任务2', details: '详情2', priority: 2, task_order: 2, compele: false },
                { title: '任务3', details: '详情3', priority: 3, task_order: 3, compele: false }
            ];
            
            const insertResult = await taskApi.insertTasks(tasksToInsert);
            console.log('批量插入结果:', insertResult);

        } catch (error) {
            console.error('高级操作失败:', error.message);
        }
    }
}; 