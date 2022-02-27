import { ControllerBaseProxy } from 'bizify';
import { RUN_TYPE_CLIENT, RUN_TYPE_SERVER } from '../../common';
import { configurationService } from '../../services/configuration.service';
import { securityService } from '../../services/security.service';
import { ErrorInfo, SecurityInfo } from '../../typings';

type Configuration = { config: any; metadata: any };

type Data = {
  loaded: boolean;
  config: any;
  isFormDirty: boolean;
  isSaveSuccess: boolean;
  errorInfo: {
    load: ErrorInfo | null;
    mutate: ErrorInfo | null;
  };
};

export class ConfigurationCtrl extends ControllerBaseProxy<Data> {
  $data(): Data {
    return {
      loaded: false,
      config: null,
      isFormDirty: false,
      isSaveSuccess: false,
      errorInfo: {
        load: null,
        mutate: null,
      },
    };
  }

  services = {
    querySecurityConfig: this.$buildService<SecurityInfo>(securityService.query.bind(securityService)),
    createSecurityConfig: this.$buildService<SecurityInfo>(securityService.create.bind(securityService)),
    queryConfig: this.$buildService<Configuration>(configurationService.query.bind(configurationService)),
    createConfig: this.$buildService<Configuration>(configurationService.create.bind(configurationService)),
    modifyConfig: this.$buildService<Configuration>(configurationService.modify.bind(configurationService)),
  };

  init = async () => {
    try {
      const { config } = await this.services.queryConfig.execute();

      if (!this.isConfigValid(config)) {
        throw Error('invalid configuration');
      }

      this.data.config = config;

      const res = await this.services.querySecurityConfig.execute();
      this.updateConfig(res);

    } catch (err: any) {
      this.data.errorInfo.load = { message: err.message };
    } finally {
      this.data.loaded = true;
    }
  }

  private isConfigValid = (config: any) => {
    if (RUN_TYPE_CLIENT && config.tls_key) {
      return false;
    }
    if (RUN_TYPE_SERVER && config.with_basic_auth) {
      return false;
    }
    return true;
  };

  private updateConfig = (securityConfig: SecurityInfo) => {
    this.data.config.tls_cert = securityConfig?.certificate || null;

    if (RUN_TYPE_SERVER) {
      this.data.config.tls_key = securityConfig?.private_key || null;
    }
  };

  handleItemChange = (key: string, value: any) => {
    let normalized = value;
    if (value === undefined) {
      normalized = null;
    }
    this.data.config[key] = normalized;
    this.data.isFormDirty = true;
  };

  handleCreateConfig = async () => {
    try {
      this.data.errorInfo.mutate = null;
      const res = await this.services.createConfig.execute();
      this.data.config = res;
    } catch (err: any) {
      this.data.errorInfo.mutate = { message: err.message };
    }
  };

  handleCreateCertKey = async () => {
    try {
      this.data.errorInfo.mutate = null;

      const hostname = prompt('Please specify hostname:', 'localhost');
      const res = await this.services.createSecurityConfig.execute(hostname);

      this.updateConfig(res);
    } catch (err: any) {
      this.data.errorInfo.mutate = { message: err.message };
    }
  };

  handleSaveConfig = async () => {
    try {
      this.data.isSaveSuccess = false;
      this.data.errorInfo.mutate = null;

      await this.services.modifyConfig.execute(this.data.config);

      this.data.isSaveSuccess = true;
      this.data.isFormDirty = false;
    } catch (err: any) {
      this.data.errorInfo.mutate = { message: err.message };
    }
  };
}
