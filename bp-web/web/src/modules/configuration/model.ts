import { ControllerBaseProxy } from 'bizify';
import { RUN_TYPE_CLIENT, RUN_TYPE_SERVER } from '../../common';
import { configService } from '../../services/config.service';
import { ErrorInfo } from '../../typings';

type Configuration = {
  config: any;
  metadata: any;
};

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
    queryConfig: this.$buildService<Configuration>(configService.query.bind(configService)),
    createConfig: this.$buildService<Configuration>(configService.create.bind(configService)),
    createTLSConfig: this.$buildService<void>(configService.create_tls_config.bind(configService)),
    modifyConfig: this.$buildService<void>(configService.modify.bind(configService)),
  };

  init = async () => {
    try {
      const { config } = await this.services.queryConfig.execute();

      if (!this.isConfigValid(config)) {
        throw Error('invalid configuration');
      }

      this.data.config = config;
    } catch (err: any) {
      this.data.errorInfo.load = { message: err.message };
    } finally {
      this.data.loaded = true;
    }
  }

  private isConfigValid = (config: any) => {
    if (RUN_TYPE_CLIENT && config?.tls_key) {
      return false;
    }
    if (RUN_TYPE_SERVER && config?.with_basic_auth) {
      return false;
    }
    return true;
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
      const { config } = await this.services.createConfig.execute();
      this.data.config = config;
    } catch (err: any) {
      this.data.errorInfo.mutate = { message: err.message };
    }
  };

  handleCreateTLSConfig = async () => {
    try {
      this.data.errorInfo.mutate = null;

      const hostname = prompt('Please specify hostname:', 'localhost');
      await this.services.createTLSConfig.execute(hostname);

      this.init();
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
