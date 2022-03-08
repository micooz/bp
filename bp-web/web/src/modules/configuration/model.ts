import { ControllerBaseProxy } from 'bizify';
import { RUN_TYPE_CLIENT, RUN_TYPE_SERVER } from '../../common';
import { configService } from '../../services/config.service';
import { ErrorInfo } from '../../typings';
import { isValidJson } from '../../utils';

type Configuration = {
  file_path: string;
  config: any;
  metadata: any;
};

type Data = {
  loaded: boolean;
  file_path: string;
  config: any;
  configString: string;
  isFormDirty: boolean;
  isShowCode: boolean;
  isSaveSuccess: boolean;
  errorInfo: {
    load: ErrorInfo | null;
    mutate: ErrorInfo | null;
    code: ErrorInfo | null;
  };
};

export class ConfigurationCtrl extends ControllerBaseProxy<Data> {
  $data(): Data {
    return {
      loaded: false,
      file_path: '',
      config: null,
      configString: '',
      isFormDirty: false,
      isShowCode: false,
      isSaveSuccess: false,
      errorInfo: {
        load: null,
        mutate: null,
        code: null,
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
      const { file_path, config } = await this.services.queryConfig.execute();

      if (!this.isConfigValid(config)) {
        throw Error('invalid configuration');
      }

      this.data.file_path = file_path;
      this.data.config = config;
      this.data.configString = this.stringify();
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

  private stringify = () => {
    return JSON.stringify(this.data.config, null, 2);
  };

  handleItemChange = (key: string, value: any) => {
    let normalized = value;
    if (value === undefined) {
      normalized = null;
    }
    this.data.config[key] = normalized;
    this.data.configString = this.stringify();
    this.data.isFormDirty = true;
  };

  handleConfigChange = (value: string) => {
    this.data.configString = value;
  };

  handleCreateConfig = async () => {
    try {
      this.data.errorInfo.mutate = null;
      const { config } = await this.services.createConfig.execute();
      this.data.config = config;
      this.data.configString = this.stringify();
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

      await this.services.modifyConfig.execute({
        modify_type: 'config',
        content: JSON.stringify(this.data.config, null, 2),
      });

      this.data.isSaveSuccess = true;
      this.data.isFormDirty = false;
    } catch (err: any) {
      this.data.errorInfo.mutate = { message: err.message };
    }
  };

  handleShowCodeClick = () => {
    const isShowCode = !this.data.isShowCode;

    if (!isShowCode && !isValidJson(this.data.configString)) {
      this.data.errorInfo.code = { message: 'Invalid JSON format' };
      return;
    }

    if (!isShowCode) {
      this.data.errorInfo.code = null;
      this.data.config = JSON.parse(this.data.configString);
      this.data.isFormDirty = true;
    }

    this.data.isShowCode = isShowCode;
  };
}
