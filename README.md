# RID

## Türkçe

Bu proje, **generative AI dan yararlanılarak** olusturulan bi init projesidir

- `example.json` örnek JSON yapısını gösterir.
- Servis dosyaları hedef sisteme `/etc/rid/services/` içine yerleştirilmelidir.
- Çok satırlı komutlar çalıştırmak için bir `.sh` scripti kullanmanız gerekir.

### Örnek JSON yapısı

```json
{
    "command": "zsh",
    "args": ["-i"]
}
```

### Hizmet dosyaları

`services/` dizini altındaki servis tanımları, hedef cihazdaki `/etc/rid/services/` dizinine kopyalanmalıdır.

### Script kullanımı

Çok satırlı komut çalıştırmak için bir `.sh` scripti oluşturun ve çalıştırılabilir hale getirin:

```bash
chmod +x run.sh
```

## English

This project use **generative AI** for asistance.

- `example.json` shows the sample JSON structure.
- Service files should be placed on the target device under `/etc/rid/services/`.
- Use a `.sh` script for running multiline commands.

### Sample JSON structure

```json
{
    "command": "zsh",
    "args": ["-i"]
}
```

### Service files

Service definitions under the `services/` folder should be copied to `/etc/rid/services/` on the target device.

### Script usage

Create a `.sh` script for multiline commands and make it executable:

```bash
chmod +x run.sh
```
