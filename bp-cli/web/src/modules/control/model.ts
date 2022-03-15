import { ControllerBaseProxy } from 'bizify';
import { serviceService } from '../../services/service.service';
import { ServiceInfo } from '../../typings';

type Data = {
  online: boolean;
  services: ServiceInfo[];
};

export class ControlCtrl extends ControllerBaseProxy<Data> {
  $data(): Data {
    return {
      online: false,
      services: [],
    };
  }

  services = {
    query: this.$buildService(serviceService.query.bind(serviceService)),
    start: this.$buildService(serviceService.start.bind(serviceService)),
    stop: this.$buildService(serviceService.stop.bind(serviceService)),
  };

  init = async () => {
    const services = (await this.services.query.execute() as any as ServiceInfo[]) || [];
    this.data.services = services;
    this.data.online = services.length > 0;
  };

  toggleService = async () => {
    if (this.data.online) {
      await this.services.stop.execute();
      this.data.services = [];
    } else {
      const services = await this.services.start.execute() as any as ServiceInfo[];
      this.data.services = services;
    }
    this.data.online = !this.data.online;
  };
}
