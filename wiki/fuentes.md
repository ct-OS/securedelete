# Fuentes y referencias

Documentación consultada para esta wiki. Las referencias históricas de los estándares
proceden de los textos oficiales cuando son públicos y, cuando no lo son (BMB21, "BSI 9",
"NSA 130-1"), se marca expresamente como atribución de terceros no verificable en fuente
primaria.

## Estándares y manuales

- **NCSC-TG-025**, *A Guide to Understanding Data Remanence in Automated Information
  Systems*, National Computer Security Center, versión 2, sept-1991 (Rainbow Series).
- **DoD 5220.22-M / NISPOM**, ed. ene-1995 (cap. 8, párr. 8-306, *Clearing and
  Sanitization Matrix*); reeditado 28-feb-2006 sin la matriz; cambios 2013, 2016;
  32 CFR Part 117, 2021.
- **NSA/CSS Policy Manual 9-12**, *Storage Device Sanitization*, sucesor del Manual 130-2.
- **AR 25-2**, *Information Assurance*, US Army, 2003 (sustituye a AR 380-19).
- **AR 380-19**, *Information Systems Security*, US Army, ed. 27-feb-1998, apéndice de borrado.
- **NAVSO P-5239-26**, *Remanence Security Guidebook*, Module 26, Naval Information Systems
  Management Center, sept-1993.
- **AFSSI-5020**, *Remanence Security*, US Air Force, 1996.
- **NSA/CSS Manual 130-2**, *Media Declassification and Destruction Manual*, nov-2000.
- **DOE M 205.1-2**, *Departmental Manual for Personnel Security and Information Security*, 26-jun-2005.
- **NIST SP 800-88**, *Guidelines for Media Sanitization*, sep-2006; Rev 1 dic-2014; Rev 2 26-sep-2025.
- **HMG Infosec Standard No. 5**, *Secure Sanitisation* (CESG/NCSC), Issue 4.0 abr-2011 y siguientes.
- **RCMP TSSIT**, *Technical Security Standard for Information Technology*, 1995 (BT32-36/6-1995E);
  lista de productos aprobados **B2-002**, may-2009.
- **VSITR**, BMI de Alemania, 1-sep-1998, con **BSI Hinweisblatt Nr. 11** (21-nov-1999);
  derogado 1-jun-2006.
- **GOST R 50739-95**, estándar estatal ruso, 1995.
- **BMB21-2019**, Bundeswehr (documento interno; secuencia divulgada por nwipe).
- **BSI IT-Grundschutz**, módulo M 2.167 (2–3 pasadas) y CON.6 (una pasada PRNG).

## Artículos académicos

- Peter Gutmann, *Secure Deletion of Data from Magnetic and Solid-State Memory*,
  6.º Simposio de Seguridad de la USENIX, San José, julio 1996.
- Bruce Schneier, *Applied Cryptography*, 1994.
- Roy Pfitzner, *Sicheres Löschen von Dateien und Datenträgern*, comisionado de protección
  de datos de Brandeburgo, 2003.

## Herramientas

- GNU coreutils `shred` (Colin Plumb; cambio a 3 pasadas en coreutils 7.1, feb-2009).
- `wipe` (Berke Durak, 1998–2016).
- `ya-wipe` 2.3.1 (Thomas M. Vier Jr., 1-nov-2009).
- `scrub` (Jim Garlick, LLNL, UCRL-CODE-2003-006).
- `DBAN` (Darik Horn, betas 2003, 1.0.0 27-may-2003).
- `nwipe` (Martijn van Brummelen, repo 2013, primer release 20-oct-2014).
- `Eraser` (Sami Tolvanen ~1997; Heidi Computers).
- `BleachBit` (Andrew Ziem, 24-dic-2008).
- `secure-delete` / `srm` (van Hauser, THC).

## Artículos divulgativos

- Artículo *Secure Deletion and Data Remanence* (Wikipedia en inglés), que documenta el
  folklore del "DoD de 3 pasadas" y la retirada de la sobrescritura del NISPOM en 2006.
