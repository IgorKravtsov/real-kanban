const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3001';
const API_KEY = import.meta.env.VITE_API_KEY || '';

export interface Project {
  id: number;
  name: string;
  sort_order: number;
  created_at: string;
}

export interface Column {
  id: number;
  project_id: number;
  name: string;
  sort_order: number;
  created_at: string;
}

export interface Task {
  id: number;
  project_id: number;
  column_id: number;
  title: string;
  description?: string | null;
  priority?: string | null;
  sort_order: number;
  source_tag?: string | null;
  created_at?: string | null;
}

export interface ColumnWithTasks extends Column {
  tasks: Task[];
}

export interface ProjectWithDetails extends Project {
  columns: ColumnWithTasks[];
}

export interface Subtask {
  id: number;
  task_id: number;
  title: string;
  completed: boolean;
  created_at: string;
}

export interface Tag {
  id: number;
  name: string;
  color: string;
  created_at: string;
}

export interface LinkedPath {
  id: number;
  project_id: number;
  path: string;
  hostname: string | null;
  default_column_id: number | null;
  created_at: string;
}

class ApiError extends Error {
  status: number;
  data?: unknown;

  constructor(message: string, status: number, data?: unknown) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.data = data;
  }
}

async function fetchApi<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    ...(API_KEY && { 'X-API-Key': API_KEY }),
    ...options.headers,
  };

  const response = await fetch(`${API_URL}/api${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    const errorData = await response.json().catch(() => null);
    throw new ApiError(
      errorData?.error || 'API request failed',
      response.status,
      errorData
    );
  }

  if (response.status === 204 || response.headers.get('content-length') === '0') {
    return undefined as T;
  }

  return response.json();
}

export const api = {
  projects: {
    list: () => fetchApi<Project[]>('/projects'),
    
    get: (id: number) => fetchApi<ProjectWithDetails>(`/projects/${id}`),
    
    create: (name: string) =>
      fetchApi<Project>('/projects', {
        method: 'POST',
        body: JSON.stringify({ name }),
      }),
    
    update: (id: number, name: string) =>
      fetchApi<Project>(`/projects/${id}`, {
        method: 'PUT',
        body: JSON.stringify({ name }),
      }),
    
    delete: (id: number) =>
      fetchApi<void>(`/projects/${id}`, {
        method: 'DELETE',
      }),
    
    reorder: (items: Array<{ id: number; sort_order: number }>) =>
      fetchApi<void>('/projects/reorder', {
        method: 'PUT',
        body: JSON.stringify(items),
      }),
  },

  tasks: {
    list: (projectId?: number) => {
      const query = projectId ? `?project_id=${projectId}` : '';
      return fetchApi<Task[]>(`/tasks${query}`);
    },
    
    get: (id: number) => fetchApi<Task>(`/tasks/${id}`),
    
    create: (data: {
      project_id: number;
      column_id: number;
      title: string;
      description?: string;
      priority?: string;
      sort_order?: number;
    }) =>
      fetchApi<Task>(`/projects/${data.project_id}/tasks`, {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    
    update: (
      id: number,
      data: {
        title?: string;
        description?: string;
        priority?: string;
        column_id?: number;
        sort_order?: number;
      }
    ) =>
      fetchApi<Task>(`/tasks/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
    
    delete: (id: number) =>
      fetchApi<void>(`/tasks/${id}`, {
        method: 'DELETE',
      }),
    
    move: (id: number, data: { column_id: number; sort_order: number }) =>
      fetchApi<Task>(`/tasks/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
  },

  columns: {
    create: (projectId: number, name: string) =>
      fetchApi<Column>(`/projects/${projectId}/columns`, {
        method: 'POST',
        body: JSON.stringify({ name }),
      }),
    
    update: (id: number, data: { name?: string; sort_order?: number }) =>
      fetchApi<Column>(`/columns/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
    
    delete: (id: number) =>
      fetchApi<void>(`/columns/${id}`, {
        method: 'DELETE',
      }),
    
    reorder: (projectId: number, items: Array<{ id: number; sort_order: number }>) =>
      fetchApi<void>(`/projects/${projectId}/columns/reorder`, {
        method: 'PUT',
        body: JSON.stringify(items),
      }),
  },

  subtasks: {
    list: (taskId: number) => fetchApi<Subtask[]>(`/tasks/${taskId}/subtasks`),
    
    create: (taskId: number, title: string) =>
      fetchApi<Subtask>('/subtasks', {
        method: 'POST',
        body: JSON.stringify({ task_id: taskId, title }),
      }),
    
    update: (id: number, data: { title?: string; completed?: boolean }) =>
      fetchApi<Subtask>(`/subtasks/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
    
    delete: (id: number) =>
      fetchApi<void>(`/subtasks/${id}`, {
        method: 'DELETE',
      }),
  },

  tags: {
    list: () => fetchApi<Tag[]>('/tags'),
    
    create: (name: string, color: string) =>
      fetchApi<Tag>('/tags', {
        method: 'POST',
        body: JSON.stringify({ name, color }),
      }),
    
    update: (id: number, data: { name?: string; color?: string }) =>
      fetchApi<Tag>(`/tags/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
    
    delete: (id: number) =>
      fetchApi<void>(`/tags/${id}`, {
        method: 'DELETE',
      }),
  },

  linkedPaths: {
    list: (projectId: number) => fetchApi<LinkedPath[]>(`/projects/${projectId}/linked-paths`),
    
    delete: (id: number) =>
      fetchApi<void>(`/linked-paths/${id}`, {
        method: 'DELETE',
      }),
  },
};
