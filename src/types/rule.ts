export interface Rule {
  id: string;
  softwareId: string;
  name: string;
  type: string;
  filePath: string;
  content: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}
