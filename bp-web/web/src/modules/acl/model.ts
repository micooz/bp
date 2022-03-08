import { ControllerBaseProxy } from 'bizify';
import { configService } from '../../services/config.service';

type Data = {
  file_path: string;
  content: string;
};

export class AclCtrl extends ControllerBaseProxy<Data> {
  $data(): Data {
    return {
      file_path: '',
      content: '',
    };
  }

  services = {
    query: this.$buildService<{ file_path: string; content: string }>(configService.query_acl.bind(configService)),
    modify: this.$buildService<void>(configService.modify.bind(configService)),
  };

  init = async () => {
    const { file_path, content } = await this.services.query.execute();
    this.data.file_path = file_path;
    this.data.content = content;
  };

  handleInputChange = (value: string) => {
    this.data.content = value;
  };

  handleSave = async () => {
    await this.services.modify.execute({
      modify_type: 'acl',
      content: this.data.content,
    });
  };
}
