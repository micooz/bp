import { ControllerBaseProxy } from 'bizify';
import { ConfigurationService } from '../../services/configuration.service';
import { Configuration } from '../../typings';

const service = new ConfigurationService();

interface Data {
  loaded: boolean;
  config: Configuration | null;
}

export class ConfigurationCtrl extends ControllerBaseProxy<Data> {
  $data(): Data {
    return {
      loaded: false,
      config: null,
    };
  }

  services = {
    query: this.$buildService<Configuration>(service.query.bind(service)),
    create: this.$buildService<Configuration>(service.create.bind(service)),
    modify: this.$buildService<Configuration>(service.modify.bind(service)),
  };

  init = async () => {
    try {
      const config = await this.services.query.execute();
      this.data.config = config;
    } catch (err: any) {
      // console.error(err.status);
    } finally {
      this.data.loaded = true;
    }
  }

  create = async () => {
    try {
      const config = await this.services.create.execute();
      this.data.config = config;
    } catch (err: any) {
      // console.error(err.status);
    }
  };
}
