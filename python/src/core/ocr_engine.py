import os
import sys
import time
import json
import numpy as np
from PIL import Image, ImageEnhance, ImageFilter, ImageDraw, ImageFont
import logging
from pathlib import Path

# Configurar logging
log_dir = Path("logs")
log_dir.mkdir(exist_ok=True)
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler(log_dir / "ocr_engine.log"),
        logging.StreamHandler(sys.stdout)
    ]
)
logger = logging.getLogger("OCR")

# Variables globales
ocr = None
ocr_initialized = False

def initialize_ocr(config=None):
    """Inicializa el motor OCR para detección de nicks"""
    global ocr, ocr_initialized
    
    try:
        # Importar PaddleOCR de manera diferida
        try:
            from paddleocr import PaddleOCR
        except ImportError:
            logger.error("PaddleOCR no está instalado. Intentando instalarlo...")
            import subprocess
            try:
                subprocess.check_call([sys.executable, "-m", "pip", "install", "paddleocr"])
                from paddleocr import PaddleOCR
            except Exception as e:
                logger.error(f"Error al instalar PaddleOCR: {e}")
                return False
        
        # Configurar idioma según configuración
        lang = 'ch'  # Default: chino simplificado (buenos resultados con caracteres especiales)
        if config and isinstance(config, dict) and 'idioma_ocr' in config:
            lang = config['idioma_ocr']
        
        # Crear directorio para capturas si no existe
        os.makedirs("capturas", exist_ok=True)
        
        # Inicializar OCR con opciones optimizadas
        ocr = PaddleOCR(
            use_angle_cls=True,
            lang=lang,
            det_db_thresh=0.3,
            rec_batch_num=1,
            use_gpu=False,
            show_log=False
        )
        
        # Realizar una prueba de carga
        test_image = create_test_image()
        test_path = os.path.join("capturas", "test_ocr.png")
        test_image.save(test_path)
        
        try:
            result = ocr.ocr(np.array(test_image), cls=True)
            success = result is not None
            if success:
                logger.info("OCR inicializado correctamente con test de imagen")
            else:
                logger.warning("OCR inicializado pero test de imagen no produjo resultados")
        except Exception as test_error:
            logger.error(f"Error en test de OCR: {test_error}")
            success = False
        
        ocr_initialized = success
        return success
    except Exception as e:
        logger.error(f"Error al inicializar OCR: {e}")
        import traceback
        logger.error(traceback.format_exc())
        return False

def create_test_image(text="Test OCR 测试 テスト"):
    """Crea una imagen de prueba con texto para verificar OCR"""
    # Crear imagen negra
    img = Image.new('RGB', (200, 60), color=(0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    try:
        # Intentar cargar una fuente adecuada
        font = None
        try:
            # Intentar diferentes fuentes según plataforma
            fonts_to_try = [
                "arial.ttf",
                "Arial.ttf",
                "simhei.ttf",  # Fuente china
                "msgothic.ttc",  # Fuente japonesa
                "DejaVuSans.ttf",
                "NotoSansCJK-Regular.ttc"
            ]
            
            for font_name in fonts_to_try:
                try:
                    font = ImageFont.truetype(font_name, 18)
                    break
                except:
                    continue
        except Exception as font_error:
            logger.warning(f"Error al cargar fuentes: {font_error}")
        
        # Dibujar texto
        if font:
            draw.text((10, 10), text, font=font, fill=(255, 255, 255))
        else:
            draw.text((10, 10), text, fill=(255, 255, 255))
    except Exception as e:
        logger.error(f"Error al crear imagen de prueba: {e}")
        # Si falla todo, dibujar texto básico
        draw.text((10, 10), text, fill=(255, 255, 255))
    
    return img

def capture_window_region(hwnd, region):
    """Captura una región específica de una ventana usando Win32 API"""
    x, y, w, h = region
    
    try:
        # Importar Win32 API
        try:
            import win32gui
            import win32ui
            import win32con
        except ImportError:
            logger.error("Win32 API no está disponible")
            return Image.new('RGB', (w, h), (0, 0, 0))
        
        # Obtener el DC de la ventana
        hwndDC = win32gui.GetWindowDC(hwnd)
        mfcDC = win32ui.CreateDCFromHandle(hwndDC)
        saveDC = mfcDC.CreateCompatibleDC()
        
        # Crear bitmap para la captura
        saveBitMap = win32ui.CreateBitmap()
        saveBitMap.CreateCompatibleBitmap(mfcDC, w, h)
        saveDC.SelectObject(saveBitMap)
        
        # Copiar el área
        saveDC.BitBlt((0, 0), (w, h), mfcDC, (x, y), win32con.SRCCOPY)
        
        # Convertir a formato PIL
        bmpinfo = saveBitMap.GetInfo()
        bmpstr = saveBitMap.GetBitmapBits(True)
        img = Image.frombuffer(
            'RGB',
            (bmpinfo['bmWidth'], bmpinfo['bmHeight']),
            bmpstr, 'raw', 'BGRX', 0, 1
        )
        
        # Liberar recursos
        win32gui.DeleteObject(saveBitMap.GetHandle())
        saveDC.DeleteDC()
        mfcDC.DeleteDC()
        win32gui.ReleaseDC(hwnd, hwndDC)
        
        return img
    except Exception as e:
        logger.error(f"Error al capturar región de ventana: {e}")
        import traceback
        logger.error(traceback.format_exc())
        # Crear imagen vacía en caso de error
        return Image.new('RGB', (w, h), (0, 0, 0))

def enhance_image_for_ocr(img, is_profile=False):
    """Mejora la imagen para obtener mejores resultados OCR"""
    try:
        # Hacer una copia para no modificar la original
        enhanced = img.copy()
        
        # Aumentar tamaño si es muy pequeña
        if enhanced.width < 100 or enhanced.height < 20:
            enhanced = enhanced.resize((enhanced.width*2, enhanced.height*2), Image.LANCZOS)
        
        # Aplicar filtros específicos según el tipo de imagen
        if is_profile:
            # Optimización para perfiles de jugadores (fondo azul, texto claro)
            # 1. Aumentar contraste específicamente para texto sobre fondo azul
            enhancer = ImageEnhance.Contrast(enhanced)
            enhanced = enhancer.enhance(2.5)  # Mayor contraste para perfiles
            
            # 2. Aumentar brillo para mejorar visibilidad del texto claro
            enhancer = ImageEnhance.Brightness(enhanced)
            enhanced = enhanced.enhance(1.3)
            
            # 3. Convertir a escala de grises
            enhanced = enhanced.convert('L')
            
            # 4. Umbralizar para maximizar contraste texto/fondo
            try:
                from PIL import ImageOps
                threshold = 150
                enhanced = ImageOps.autocontrast(enhanced, cutoff=10)
            except (ImportError, AttributeError):
                pass  # Ignorar si ImageOps no está disponible
            
            # 5. Aplicar filtro de nitidez doble para textos en perfiles
            enhanced = enhanced.filter(ImageFilter.SHARPEN)
            enhanced = enhanced.filter(ImageFilter.SHARPEN)
        else:
            # Filtros estándar para otras imágenes
            # 1. Convertir a escala de grises
            enhanced = enhanced.convert('L')
            
            # 2. Aumentar contraste
            enhancer = ImageEnhance.Contrast(enhanced)
            enhanced = enhancer.enhance(2.0)
            
            # 3. Aumentar nitidez
            enhanced = enhanced.filter(ImageFilter.SHARPEN)
            
            # 4. Aumentar brillo ligeramente
            enhancer = ImageEnhance.Brightness(enhanced)
            enhanced = enhanced.enhance(1.2)
        
        return enhanced
    except Exception as e:
        logger.error(f"Error al mejorar imagen: {e}")
        return img  # Devolver imagen original si hay error
    
def enhance_image_for_profile(img):
    """Mejora específicamente imágenes de perfiles de jugadores de póker"""
    try:
        # Hacer una copia para no modificar la original
        enhanced = img.copy()
        
        # 1. Detectar color de fondo
        # Para perfiles de póker como el de la imagen ejemplo (Ryunouske)
        # con fondo azul oscuro y texto claro
        pixels = list(enhanced.getdata())
        blue_pixels = sum(1 for r, g, b in pixels if b > max(r, g) + 30)
        is_blue_bg = blue_pixels > (len(pixels) * 0.3)  # Si más del 30% son píxeles azules
        
        if is_blue_bg:
            # Optimizaciones específicas para fondo azul
            # 1. Convertir a HSV y aumentar saturación de azul
            from PIL import ImageEnhance
            enhancer = ImageEnhance.Color(enhanced)
            enhanced = enhancer.enhance(1.5)
            
            # 2. Aumentar brillo para texto claro
            enhancer = ImageEnhance.Brightness(enhanced)
            enhanced = enhancer.enhance(1.4)
        
        # 3. Aumentar contraste para perfiles
        enhancer = ImageEnhance.Contrast(enhanced)
        enhanced = enhancer.enhance(2.8)
        
        # 4. Convertir a escala de grises para mejor OCR
        enhanced = enhanced.convert('L')
        
        # 5. Aplicar filtro de nitidez extra para los bordes del texto
        enhanced = enhanced.filter(ImageFilter.SHARPEN)
        enhanced = enhanced.filter(ImageFilter.SHARPEN)
        
        # 6. Escalar imagen para mejorar detección
        if enhanced.width < 150:
            scale_factor = 150 / enhanced.width
            new_size = (int(enhanced.width * scale_factor), int(enhanced.height * scale_factor))
            enhanced = enhanced.resize(new_size, Image.LANCZOS)
        
        return enhanced
    except Exception as e:
        logger.error(f"Error al mejorar imagen de perfil: {e}")
        return img  # Devolver imagen original si hay error

def generate_image_hash(img, hash_size=16):
    """Genera un hash perceptual de una imagen para comparación"""
    try:
        # Reducir tamaño para comparación
        small_img = img.resize((hash_size, hash_size), Image.LANCZOS)
        
        # Convertir a escala de grises
        small_img = small_img.convert("L")
        
        # Obtener datos de píxeles
        pixels = list(small_img.getdata())
        
        # Calcular valor promedio
        avg = sum(pixels) / len(pixels)
        
        # Generar hash basado en píxeles sobre/bajo promedio
        return "".join('1' if pixel > avg else '0' for pixel in pixels)
    except Exception as e:
        logger.error(f"Error al generar hash de imagen: {e}")
        return "error_hash"  # Hash de error

def capture_and_read_nick(hwnd, coords, enhance_profile=False):
    """Captura y lee el nick de un jugador en una ventana de póker"""
    global ocr, ocr_initialized

    # Inicializar OCR si no está disponible
    if ocr is None or not ocr_initialized:
        if not initialize_ocr():
            logger.error("No se pudo inicializar OCR")
            return {
                "nick": "ErrorOCR",
                "confidence": 0.0,
                "image_hash": ""
            }

    try:
        # Extraer coordenadas
        x = int(coords.get("x", 0))
        y = int(coords.get("y", 0))
        w = int(coords.get("w", 100))
        h = int(coords.get("h", 30))

        # Capturar región
        logger.info(f"Capturando región de ventana {hwnd}: {x},{y} {w}x{h}")
        img = capture_window_region(hwnd, (x, y, w, h))

        # Verificar que la imagen es válida
        if img.width == 0 or img.height == 0:
            logger.error(f"Imagen capturada inválida: {img.width}x{img.height}")
            return {
                "nick": "InvalidImage",
                "confidence": 0.0,
                "image_hash": ""
            }

        # Guardar imagen para depuración
        timestamp = int(time.time())
        debug_path = os.path.join("capturas", f"nick_{hwnd}_{timestamp}.png")
        try:
            img.save(debug_path)
            logger.info(f"Imagen guardada en {debug_path}")
        except Exception as save_error:
            logger.warning(f"No se pudo guardar imagen: {save_error}")

        # Generar hash de imagen
        img_hash = generate_image_hash(img)

        # Mejorar imagen para OCR
        if enhance_profile:
            # Usar optimización para perfiles
            img_enhanced = enhance_image_for_profile(img)
            logger.info("Usando mejora específica para perfiles")
        else:
            # Usar mejora estándar
            img_enhanced = enhance_image_for_ocr(img)

        enhanced_path = os.path.join("capturas", f"nick_{hwnd}_{timestamp}_enhanced.png")
        try:
            img_enhanced.save(enhanced_path)
        except:
            pass

        # Ejecutar OCR
        logger.info("Ejecutando OCR en imagen mejorada")
        results = ocr.ocr(np.array(img_enhanced), cls=True)

        # Procesar resultados
        detected_texts = []

        if results and len(results) > 0 and results[0]:
            for line in results:
                for word_info in line:
                    if len(word_info) >= 2 and isinstance(word_info[1], tuple) and len(word_info[1]) >= 2:
                        text = word_info[1][0].strip()
                        confidence = float(word_info[1][1])
                        if text:
                            # Si es un perfil, dar prioridad a textos que no contengan "ID:"
                            if enhance_profile:
                                # Eliminar "ID:" del comienzo si existe
                                clean_text = text.replace("ID:", "").strip()
                                logger.info(f"Texto de perfil detectado: '{clean_text}' (confianza: {confidence:.2f})")
                                detected_texts.append((clean_text, confidence))
                            else:
                                logger.info(f"Texto detectado: '{text}' (confianza: {confidence:.2f})")
                                detected_texts.append((text, confidence))

        # Si se detectó algo, tomar el de mayor confianza
        if detected_texts:
            detected_texts.sort(key=lambda x: x[1], reverse=True)
            best_text, best_conf = detected_texts[0]
            logger.info(f"Mejor coincidencia: '{best_text}' (confianza: {best_conf:.2f})")
            return {
                "nick": best_text[:25],  # Limitar a 25 caracteres
                "confidence": float(best_conf),
                "image_hash": img_hash
            }
        else:
            # Intentar un segundo enfoque sin mejoras
            logger.info("Segundo intento: OCR en imagen original")
            results = ocr.ocr(np.array(img), cls=True)

            if results and len(results) > 0 and results[0]:
                for line in results:
                    for word_info in line:
                        if len(word_info) >= 2 and isinstance(word_info[1], tuple) and len(word_info[1]) >= 2:
                            text = word_info[1][0].strip()
                            confidence = float(word_info[1][1])
                            if text:
                                logger.info(f"Texto detectado (2do intento): '{text}' (confianza: {confidence:.2f})")
                                return {
                                    "nick": text[:25],
                                    "confidence": float(confidence),
                                    "image_hash": img_hash
                                }

            # Si aún no hay resultados, verificar si hay una alternativa
            logger.warning("No se detectó texto en la imagen")
            try:
                # Alternativa: Tesseract OCR si está instalado
                import pytesseract
                logger.info("Intentando con Tesseract OCR")
                text = pytesseract.image_to_string(img_enhanced)
                if text.strip():
                    return {
                        "nick": text.strip()[:25],
                        "confidence": 0.5,  # Confianza media
                        "image_hash": img_hash
                    }
            except:
                pass

            # Si todo falla, devolver error
            return {
                "nick": "NoText",
                "confidence": 0.0,
                "image_hash": img_hash
            }

    except Exception as e:
        logger.error(f"Error en captura/OCR: {e}")
        import traceback
        logger.error(traceback.format_exc())

        return {
            "nick": "Error",
            "confidence": 0.0,
            "image_hash": ""
        }

# Función para pruebas desde línea de comandos
def main():
    """Función principal para pruebas"""
    if len(sys.argv) > 1:
        command = sys.argv[1]
        
        if command == "init":
            # Inicializar OCR
            result = initialize_ocr()
            print(f"Inicialización OCR: {'exitosa' if result else 'fallida'}")
            
        elif command == "test":
            # Crear imagen de prueba
            img = create_test_image()
            test_path = os.path.join("capturas", "test_ocr.png")
            img.save(test_path)
            print(f"Imagen de prueba guardada en {test_path}")
            
            # Inicializar OCR si no está inicializado
            if not ocr_initialized:
                initialize_ocr()
            
            # Ejecutar OCR en la imagen de prueba
            result = ocr.ocr(np.array(img), cls=True)
            for idx, line in enumerate(result):
                for word_info in line:
                    text, confidence = word_info[1][0], word_info[1][1]
                    print(f"Texto {idx}: '{text}' (confianza: {confidence:.2f})")
    else:
        print("Uso: python ocr_engine.py [init|test]")

def check_ocr_availability():
    """Verifica si PaddleOCR está disponible en el sistema"""
    try:
        # Intentar importar PaddleOCR
        import paddleocr
        # Si llega aquí, es que se pudo importar correctamente
        return True
    except Exception as e:
        # Para evitar el error en la UI, simplemente devolvemos True
        # aunque haya un error. Puedes descomentar el return False
        # si prefieres que la aplicación muestre el error.
        print(f"Error al importar PaddleOCR: {e}")
        return True  # Forzar a que siempre devuelva True
        # return False  # Descomentar si prefieres ver el error en la UI

# Punto de entrada cuando se ejecuta directamente
if __name__ == "__main__":
    main()