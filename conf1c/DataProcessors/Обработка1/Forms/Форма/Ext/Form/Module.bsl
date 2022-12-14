
&НаКлиенте
Процедура Тест1(Команда)
	Тест1НаСервере(ИмяФайла);
КонецПроцедуры

&НаСервереБезКонтекста
Процедура Тест1НаСервере(ИмяФайла)
	
	Начало = ТекущаяУниверсальнаяДатаВМиллисекундах();
	
	Если Не ПодключитьВнешнююКомпоненту(ИмяФайла, "Test", ТипВнешнейКомпоненты.Native, ТипПодключенияВнешнейКомпоненты.НеИзолированно) Тогда 
		Сообщить("Не удалось подключить");
		Возврат;
	КонецЕсли;
	
	Сообщить("Подключена");
	ОбъектКомпоненты = Новый ("AddIn.Test.Class1");
	Test = ОбъектКомпоненты.Test;
	Конец = ТекущаяУниверсальнаяДатаВМиллисекундах();
	Сообщить(СтрШаблон("Test: %1", Test));
	Сообщить(СтрШаблон("Длительность: %1", Конец - Начало));
	
КонецПроцедуры


&НаКлиенте
Процедура Тест2(Команда)
	Тест2НаСервере(ИмяФайла);
КонецПроцедуры

&НаСервереБезКонтекста
Процедура Тест2НаСервере(ИмяФайла) 
	
	Начало = ТекущаяУниверсальнаяДатаВМиллисекундах();
	
	Попытка
		ОбъектКомпоненты = Новый ("AddIn.Test.Class1");
	Исключение
		Если Не ПодключитьВнешнююКомпоненту(ИмяФайла, "Test", ТипВнешнейКомпоненты.Native, ТипПодключенияВнешнейКомпоненты.НеИзолированно) Тогда 
			ВызватьИсключение "Не удалось подключить";
		КонецЕсли;
		ОбъектКомпоненты = Новый ("AddIn.Test.Class1");
	КонецПопытки;
	
	ОбъектКомпоненты.PropI32 = 123;
	Если ОбъектКомпоненты.PropI32 <> 123 Тогда
		ВызватьИсключение "Не удалось установить значение PropI32";
	КонецЕсли; 
	
	ОбъектКомпоненты.PropF64 = 456.789;
	Если ОбъектКомпоненты.PropF64 <> 456.789 Тогда
		ВызватьИсключение "Не удалось установить значение PropF64";
	КонецЕсли;
	
	ОбъектКомпоненты.PropBool = Истина;
	Если ОбъектКомпоненты.PropBool <> Истина Тогда
		ВызватьИсключение "Не удалось установить значение PropBool";
	КонецЕсли;
	
	Date = ТекущаяДатаСеанса();
	ОбъектКомпоненты.PropDate = Date;
	Если ОбъектКомпоненты.PropDate <> Date Тогда
		ВызватьИсключение "Не удалось установить значение PropDate";
	КонецЕсли;
	
	ОбъектКомпоненты.PropStr = "Привет!";
	Если ОбъектКомпоненты.PropStr <> "Привет!" Тогда
		ВызватьИсключение "Не удалось установить значение PropStr";
	КонецЕсли;
	
	Blob = ПолучитьДвоичныеДанныеИзСтроки("Привет!");
	ОбъектКомпоненты.PropBlob = Blob;
	Если ОбъектКомпоненты.PropBlob <> Blob Тогда
		ВызватьИсключение "Не удалось установить значение PropBlob";
	КонецЕсли;
	
	Если ОбъектКомпоненты.Method1("11", "22", "33") <> "112233" Тогда
		ВызватьИсключение "Не удалось установить значение Method1";
	КонецЕсли;
	
	Конец = ТекущаяУниверсальнаяДатаВМиллисекундах();
	Сообщить(СтрШаблон("Длительность: %1", Конец - Начало));
	
КонецПроцедуры
