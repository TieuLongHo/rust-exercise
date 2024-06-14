/*
  Flash size is determined by installed Nordic SoftDevice (SD) and Adafruit
  nRF52 bootloader. The following settings are for SD S140 v6.1.1, requiring
  152 KiB (=0x26000) flash storage and 8 bytes of RAM (SD disabled!), according
  to the release notes. The Adafruit bootloader starts at 0xF4000 and requires
  40 KiB of flash storage and no RAM when the application is running.

  For details, refer to:
  https://infocenter.nordicsemi.com/topic/sds_s140/SDS/s1xx/mem_usage/mem_resource_map_usage.html
  https://github.com/adafruit/Adafruit_nRF52_Bootloader/blob/0.3.0/src/linker/nrf52840_s140_v6.ld
*/
MEMORY
{
  FLASH (rx) : ORIGIN = 0x00026000, LENGTH = 824K    /* 0xF4000-0x26000 */
  RAM (rwx)  : ORIGIN = 0x20000008, LENGTH = 256K-8
}
