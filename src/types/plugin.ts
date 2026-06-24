export interface Plugin {
  id: string;
  softwareId: string;
  name: string;
  version: string;
  author: string;
  description: string;
  installedPath: string;
  enabled: boolean;
  installedAt: string;
  lastUpdated: string;
}
