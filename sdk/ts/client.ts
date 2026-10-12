// RustNext Type-Safe API Client
import type { LoginResponse, DocumentResponse, WebhookSubscription } from './api';

export class RustNextClient {
  constructor(private baseUrl: string, private token?: string) {}

  setToken(token: string) {
    this.token = token;
  }

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(this.token ? { Authorization: `Bearer ${this.token}` } : {}),
      ...(options.headers as Record<string, string>),
    };
    const res = await fetch(`${this.baseUrl}${path}`, { ...options, headers });
    if (!res.ok) {
      throw new Error(`HTTP Error ${res.status}: ${await res.text()}`);
    }
    return res.json();
  }

  async login(usr: string, pwd: string): Promise<LoginResponse> {
    const res = await this.request<LoginResponse>('/api/v2/method/login', {
      method: 'POST',
      body: JSON.stringify({ usr, pwd }),
    });
    this.token = res.token;
    return res;
  }

  async listDocuments<T = any>(doctype: string, query?: Record<string, any>): Promise<DocumentResponse<T>> {
    const params = new URLSearchParams(query).toString();
    return this.request<DocumentResponse<T>>(`/api/v2/document/${doctype}${params ? '?' + params : ''}`);
  }

  async getDocument<T = any>(doctype: string, name: string): Promise<DocumentResponse<T>> {
    return this.request<DocumentResponse<T>>(`/api/v2/document/${doctype}/${name}`);
  }

  async createDocument<T = any>(doctype: string, doc: Partial<T>): Promise<DocumentResponse<T>> {
    return this.request<DocumentResponse<T>>(`/api/v2/document/${doctype}`, {
      method: 'POST',
      body: JSON.stringify(doc),
    });
  }

  async submitDocument<T = any>(doctype: string, name: string): Promise<DocumentResponse<T>> {
    return this.request<DocumentResponse<T>>(`/api/v2/document/${doctype}/${name}/submit`, {
      method: 'POST',
    });
  }
}
