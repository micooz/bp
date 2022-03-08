import { ControllerBaseProxy } from 'bizify';
import { serviceService } from '../../services/service.service';
import { ServiceInfo } from '../../typings';

type Data = {
  online: boolean;
  serviceInfo: ServiceInfo | null;
};

export class ControlCtrl extends ControllerBaseProxy<Data> {
  $data(): Data {
    return {
      online: false,
      serviceInfo: null,
    };
  }

  services = {
    query: this.$buildService<{ service_info: ServiceInfo }>(serviceService.query.bind(serviceService)),
    start: this.$buildService<{ service_info: ServiceInfo }>(serviceService.start.bind(serviceService)),
    stop: this.$buildService<void>(serviceService.stop.bind(serviceService)),
  };

  init = async () => {
    const { service_info } = await this.services.query.execute();
    this.data.serviceInfo = service_info;
    this.data.online = !!service_info;
  };

  toggleService = async () => {
    if (this.data.online) {
      await this.services.stop.execute();
    } else {
      const { service_info } = await this.services.start.execute();
      this.data.serviceInfo = service_info;
    }
    this.data.online = !this.data.online;
  };
}
