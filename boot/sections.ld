SECTIONS
 {
   . = 0x100000;

   .text :
    {
      *(".text")
      *(".text$")
      *(".init")
      *(".fini")
      *(".gnu.linkonce.*")
    }

    .init_array     :
    {
       PROVIDE_HIDDEN (__init_array_start = .);
       KEEP (*(SORT(.init_array.*)))
       KEEP (*(.init_array))
       PROVIDE_HIDDEN (__init_array_end = .);
    }

   .data :
    {
      *(".data")
      *(".data$")
      *(".rodata")
      ___CTOR_LIST__ = .;
      *(".ctors")
      *(".ctor")
      ___CTOR_LIST_END__ = .;
      ___DTOR_LIST__ = .;
      *(".dtors")
      *(".dtor")
      ___DTOR_LIST_END__ = .;
      *(".got")
      *(".got.plt")
      *(".eh_frame")
      *(".eh_fram")
      *(".jcr")
    }

   .setup_ap_seg ALIGN(0x10) :
    {
      ___SETUP_AP_START__ = .;
      *(".setup_ap_seg")
      *(".setup_ap_seg$")
      ___SETUP_AP_END__ = .;
    }
    .vesa_detection ALIGN(0x10):
    {
	__VESA_DETECTION_CODE_START__ = .;
	*(".vesa_detection")
	*(".vesa_detection$")
	__VESA_DETECTION_CODE_END__ = .;
    }
   .bss :
    {
      *(".bss")
      *(".bss.*")
      ___IMG_END___ = .;
    }

 }
